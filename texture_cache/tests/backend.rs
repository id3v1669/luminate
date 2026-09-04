//! End-to-end tests of the render backend through the public API:
//! record → composite → screenshot, with pixel assertions.
//!
//! The `tiny_skia` tests need no GPU. The wgpu tests are `#[ignore]`d: run them
//! locally on a machine with an adapter with
//! `cargo test -p iced_texture_cache, --include-ignored`; the CI `gpu` job
//! runs them on lavapipe.

use iced_core::Renderer as _;
use iced_core::renderer::{Headless, Quad};
use iced_core::{Color, Point, Rectangle, Size, Transformation};
use iced_texture_cache::{Backend, Record, Renderer, TextureCache, TextureRenderer};

const CANVAS: Size<u32> = Size {
    width: 8,
    height: 8,
};
const TEXTURE: Size<u32> = Size {
    width: 4,
    height: 4,
};

fn canvas() -> Rectangle {
    Rectangle::with_size(Size::new(8.0, 8.0))
}

/// RGBA of the pixel at `(x, y)` of an 8 x 8 screenshot.
fn pixel(rgba: &[u8], x: u32, y: u32) -> [u8; 4] {
    let i = ((y * CANVAS.width + x) * 4) as usize;
    [rgba[i], rgba[i + 1], rgba[i + 2], rgba[i + 3]]
}

/// Records a solid red 4 x 4 px texture into `cache` at `scale_factor`.
fn record_red(renderer: &mut Renderer, cache: &TextureCache, scale_factor: f32) -> Record {
    renderer.record(cache, TEXTURE, scale_factor, |r| {
        let logical = 4.0 / scale_factor;
        r.fill_quad(
            Quad {
                bounds: Rectangle::with_size(Size::new(logical, logical)),
                ..Quad::default()
            },
            Color::from_rgb(1.0, 0.0, 0.0),
        );
    })
}

/// Composites `cache` at (2, 2) over white and returns the screenshot.
fn composite(renderer: &mut Renderer, cache: &TextureCache, opacity: f32) -> Vec<u8> {
    renderer.reset(canvas());
    renderer.draw_cached(
        cache,
        Rectangle::new(Point::new(2.0, 2.0), Size::new(4.0, 4.0)),
        canvas(),
        Transformation::IDENTITY,
        opacity,
    );
    renderer.screenshot(CANVAS, 1.0, Color::WHITE)
}

const WHITE: [u8; 4] = [255, 255, 255, 255];

fn assert_red(px: [u8; 4]) {
    assert!(
        px[0] >= 250 && px[1] <= 3 && px[2] <= 3 && px[3] == 255,
        "pure red: {px:?}"
    );
}

#[cfg(feature = "tiny-skia")]
mod tiny_skia {
    use std::cell::Cell;

    use super::*;
    use iced_texture_cache::testing::headless_tiny_skia;

    /// Red over white at 50 %: full red, half green and blue. A BGRA swizzle
    /// bug would put the 255 in the blue channel; a premultiply bug would
    /// darken red.
    fn assert_half_red(px: [u8; 4]) {
        assert!(px[0] >= 250, "red channel: {px:?}");
        assert!(
            (120..=136).contains(&px[1]) && (120..=136).contains(&px[2]),
            "half white: {px:?}"
        );
        assert_eq!(px[3], 255, "opaque canvas: {px:?}");
    }

    #[test]
    fn the_helper_is_a_software_renderer_at_scale_one() {
        let renderer = headless_tiny_skia();
        assert_eq!(renderer.backend(), Backend::TinySkia);
        assert_eq!(renderer.scale_factor(), 1.0);
    }

    #[test]
    fn a_recorded_red_quad_composites_red_with_the_given_opacity() {
        let mut renderer = headless_tiny_skia();
        let cache = TextureCache::new();

        assert_eq!(record_red(&mut renderer, &cache, 1.0), Record::Fresh);

        let half = composite(&mut renderer, &cache, 0.5);
        assert_half_red(pixel(&half, 3, 3));
        assert_eq!(pixel(&half, 0, 0), WHITE, "outside the composite");
        assert_eq!(pixel(&half, 7, 7), WHITE, "outside the composite");

        let full = composite(&mut renderer, &cache, 1.0);
        assert_red(pixel(&full, 2, 2));
        assert_red(pixel(&full, 5, 5));
        assert_eq!(pixel(&full, 6, 6), WHITE, "the texture is 4 px wide");
    }

    #[test]
    fn nan_or_non_positive_opacity_draws_nothing_and_large_opacity_is_clamped() {
        let mut renderer = headless_tiny_skia();
        let cache = TextureCache::new();
        assert_eq!(record_red(&mut renderer, &cache, 1.0), Record::Fresh);

        assert_eq!(
            pixel(&composite(&mut renderer, &cache, f32::NAN), 3, 3),
            WHITE
        );
        assert_eq!(pixel(&composite(&mut renderer, &cache, 0.0), 3, 3), WHITE);
        assert_eq!(pixel(&composite(&mut renderer, &cache, -1.0), 3, 3), WHITE);
        assert_red(pixel(&composite(&mut renderer, &cache, 7.0), 3, 3));
    }

    #[test]
    fn a_valid_texture_is_reused_until_invalidated() {
        let mut renderer = headless_tiny_skia();
        let cache = TextureCache::new();

        assert_eq!(record_red(&mut renderer, &cache, 1.0), Record::Fresh);
        assert_eq!(record_red(&mut renderer, &cache, 1.0), Record::Reused);
        assert_eq!(cache.record_count(), 1);

        cache.invalidate();
        assert_eq!(record_red(&mut renderer, &cache, 1.0), Record::Fresh);
        assert_eq!(cache.record_count(), 2);

        // A size change re-records too, and the new content is what shows.
        let ran = Cell::new(false);
        let record = renderer.record(&cache, Size::new(5, 5), 1.0, |_| ran.set(true));
        assert_eq!(record, Record::Fresh);
        assert!(ran.get());
        assert_eq!(cache.record_count(), 3);
    }

    #[test]
    fn oversize_requests_are_uncacheable_and_never_run_the_closure() {
        let mut renderer = headless_tiny_skia();
        let cache = TextureCache::new();
        let ran = Cell::new(false);

        let per_side = renderer.record(&cache, Size::new(16_385, 1), 1.0, |_| ran.set(true));
        assert_eq!(per_side, Record::Uncacheable);

        let bytes = renderer.record(&cache, Size::new(16_384, 16_384), 1.0, |_| ran.set(true));
        assert_eq!(bytes, Record::Uncacheable, "1 GiB exceeds the 256 MiB cap");

        assert!(!ran.get(), "an uncacheable request never runs the closure");
        assert_eq!(cache.record_count(), 0);
        assert!(!cache.is_invalidated(), "the flag is consumed anyway");

        // Nothing to composite.
        assert_eq!(pixel(&composite(&mut renderer, &cache, 1.0), 3, 3), WHITE);

        // A fitting request recovers.
        assert_eq!(record_red(&mut renderer, &cache, 1.0), Record::Fresh);
        assert_red(pixel(&composite(&mut renderer, &cache, 1.0), 3, 3));
    }

    #[test]
    fn screenshot_sets_the_recording_scale_factor() {
        let mut renderer = headless_tiny_skia();
        let cache = TextureCache::new();

        assert_eq!(renderer.scale_factor(), 1.0);
        let _ = renderer.screenshot(CANVAS, 2.0, Color::WHITE);
        assert_eq!(renderer.scale_factor(), 2.0);

        // Recording at the renderer's scale works, and a scale change re-records.
        let scale = renderer.scale_factor();
        assert_eq!(record_red(&mut renderer, &cache, scale), Record::Fresh);
        assert_eq!(record_red(&mut renderer, &cache, scale), Record::Reused);
        assert_eq!(record_red(&mut renderer, &cache, 1.0), Record::Fresh);
        assert_eq!(cache.record_count(), 2);
    }

    #[test]
    fn a_nested_record_is_baked_into_the_outer_texture() {
        let mut renderer = headless_tiny_skia();
        let outer = TextureCache::new();
        let inner = TextureCache::new();
        let quad = Rectangle::with_size(Size::new(4.0, 4.0));

        let record = renderer.record(&outer, TEXTURE, 1.0, |r| {
            assert_eq!(record_red(r, &inner, 1.0), Record::Fresh);
            r.draw_cached(&inner, quad, quad, Transformation::IDENTITY, 1.0);
        });
        assert_eq!(record, Record::Fresh);
        assert_eq!((outer.record_count(), inner.record_count()), (1, 1));

        assert_red(pixel(&composite(&mut renderer, &outer, 1.0), 3, 3));
    }

    #[test]
    fn dropping_every_handle_frees_the_texture_at_the_next_frame_boundary() {
        // Observable only indirectly through the public API: a new cache with
        // a fresh identity records afresh, and the old one's texture cannot be
        // composited any more because there is no handle to name it. The
        // store-level assertion lives in `record.rs` (`store_tests`).
        let mut renderer = headless_tiny_skia();
        let cache = TextureCache::new();
        assert_eq!(record_red(&mut renderer, &cache, 1.0), Record::Fresh);
        drop(cache);
        let _ = renderer.screenshot(CANVAS, 1.0, Color::WHITE); // frame boundary
        let again = TextureCache::new();
        assert_eq!(record_red(&mut renderer, &again, 1.0), Record::Fresh);
        assert_red(pixel(&composite(&mut renderer, &again, 1.0), 3, 3));
    }
}

#[cfg(feature = "wgpu")]
mod wgpu {
    use super::*;
    use iced_core::{Font, Pixels};

    fn headless_wgpu() -> Renderer {
        iced_test::futures::futures::executor::block_on(<Renderer as Headless>::new(
            Font::DEFAULT,
            Pixels(16.0),
            Some("wgpu"),
        ))
        .expect("a GPU adapter is available")
    }

    /// Blending happens in linear light when `web-colors` is off, so only
    /// gamma-agnostic properties are asserted here.
    fn assert_half_red_any_gamma(px: [u8; 4]) {
        assert!(px[0] >= 250, "red channel: {px:?}");
        assert_eq!(px[1], px[2], "green and blue agree: {px:?}");
        assert!((100..=200).contains(&px[1]), "half-mixed: {px:?}");
    }

    #[test]
    #[ignore = "needs a GPU adapter"]
    fn a_recorded_red_quad_composites_red_with_the_given_opacity() {
        let mut renderer = headless_wgpu();
        assert_eq!(renderer.backend(), Backend::Wgpu);
        let cache = TextureCache::new();

        assert_eq!(record_red(&mut renderer, &cache, 1.0), Record::Fresh);
        assert_half_red_any_gamma(pixel(&composite(&mut renderer, &cache, 0.5), 3, 3));
        assert_red(pixel(&composite(&mut renderer, &cache, 1.0), 3, 3));
        assert_eq!(pixel(&composite(&mut renderer, &cache, 1.0), 0, 0), WHITE);
    }

    #[test]
    #[ignore = "needs a GPU adapter"]
    fn a_valid_texture_is_reused_until_invalidated() {
        let mut renderer = headless_wgpu();
        let cache = TextureCache::new();
        assert_eq!(record_red(&mut renderer, &cache, 1.0), Record::Fresh);
        assert_eq!(record_red(&mut renderer, &cache, 1.0), Record::Reused);
        cache.invalidate();
        assert_eq!(record_red(&mut renderer, &cache, 1.0), Record::Fresh);
        assert_eq!(cache.record_count(), 2);
    }

    #[test]
    #[ignore = "needs a GPU adapter"]
    fn screenshot_sets_the_recording_scale_factor() {
        let mut renderer = headless_wgpu();
        let _ = renderer.screenshot(CANVAS, 2.0, Color::WHITE);
        assert_eq!(renderer.scale_factor(), 2.0);
    }

    #[test]
    #[ignore = "needs a GPU adapter"]
    fn a_nested_record_is_baked_into_the_outer_texture() {
        let mut renderer = headless_wgpu();
        let outer = TextureCache::new();
        let inner = TextureCache::new();
        let quad = Rectangle::with_size(Size::new(4.0, 4.0));

        let record = renderer.record(&outer, TEXTURE, 1.0, |r| {
            assert_eq!(record_red(r, &inner, 1.0), Record::Fresh);
            r.draw_cached(&inner, quad, quad, Transformation::IDENTITY, 1.0);
        });
        assert_eq!(record, Record::Fresh);
        assert_red(pixel(&composite(&mut renderer, &outer, 1.0), 3, 3));
    }

    #[test]
    #[ignore = "needs a GPU adapter"]
    fn two_textures_for_one_cache_in_a_frame_draw_their_own_content() {
        // G-008: the bindings are keyed by texture, not by cache id, so a
        // re-record at another size within one frame composites the right
        // texture at each place.
        let mut renderer = headless_wgpu();
        let cache = TextureCache::new();
        renderer.reset(canvas());

        assert_eq!(record_red(&mut renderer, &cache, 1.0), Record::Fresh);
        renderer.draw_cached(
            &cache,
            Rectangle::new(Point::new(0.0, 0.0), Size::new(4.0, 4.0)),
            canvas(),
            Transformation::IDENTITY,
            1.0,
        );

        // Same cache, new size: a new texture, blue this time.
        let record = renderer.record(&cache, Size::new(2, 2), 1.0, |r| {
            r.fill_quad(
                Quad {
                    bounds: Rectangle::with_size(Size::new(2.0, 2.0)),
                    ..Quad::default()
                },
                Color::from_rgb(0.0, 0.0, 1.0),
            );
        });
        assert_eq!(record, Record::Fresh);
        renderer.draw_cached(
            &cache,
            Rectangle::new(Point::new(6.0, 6.0), Size::new(2.0, 2.0)),
            canvas(),
            Transformation::IDENTITY,
            1.0,
        );

        let shot = renderer.screenshot(CANVAS, 1.0, Color::WHITE);
        assert_red(pixel(&shot, 1, 1));
        let blue = pixel(&shot, 7, 7);
        assert!(
            blue[2] >= 250 && blue[0] <= 3 && blue[1] <= 3,
            "blue: {blue:?}"
        );
    }
}
