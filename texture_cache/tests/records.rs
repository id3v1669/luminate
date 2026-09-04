//! `record_count()` is the observable cost of a cache. These tests pin when
//! it may and may not move.
//!
//! Software backend only: the helper behind these tests is gated on
//! `tiny-skia`.

#![cfg(feature = "tiny-skia")]

use std::time::Duration;

use iced::advanced::Renderer as _;
use iced::advanced::{clipboard, renderer};
use iced::time::Instant;
use iced::widget::{column, container, pick_list, text};
use iced::{Color, Event, Padding, Point, Rectangle, Size, mouse, window};
use iced_animate::Motion;
use iced_animate::widget::shape;
use iced_test::runtime::user_interface::{self, UserInterface};
use iced_texture_cache::testing::headless_tiny_skia;
use iced_texture_cache::{Renderer, TextureCache, cached, pager};

// Tall enough for a pick list menu to open below its list (see the last test).
const SIZE: Size = Size::new(300.0, 300.0);
const FRAME: Duration = Duration::from_millis(16);

#[derive(Debug, Clone, PartialEq)]
enum Message {
    Picked(&'static str),
}

type El<'a> = iced_texture_cache::Element<'a, Message>;

struct Harness {
    renderer: Renderer,
    cache: Option<user_interface::Cache>,
    now: Instant,
    cursor: mouse::Cursor,
    messages: Vec<Message>,
}

impl Harness {
    fn new() -> Self {
        Self {
            renderer: headless_tiny_skia(),
            cache: Some(user_interface::Cache::default()),
            now: Instant::now(),
            cursor: mouse::Cursor::Unavailable,
            messages: Vec::new(),
        }
    }

    fn frame<'a>(&mut self, root: impl Into<El<'a>>, events: &[Event]) {
        self.now += FRAME;
        let cache = self.cache.take().expect("returned after every frame");
        let mut ui: UserInterface<'_, Message, iced::Theme, Renderer> =
            UserInterface::build(root, SIZE, cache, &mut self.renderer);
        let mut all = vec![Event::Window(window::Event::RedrawRequested(self.now))];
        all.extend_from_slice(events);
        let _ = ui.update(
            &all,
            self.cursor,
            &mut self.renderer,
            &mut clipboard::Null,
            &mut self.messages,
        );
        self.renderer.reset(Rectangle::with_size(SIZE));
        ui.draw(
            &mut self.renderer,
            &iced::Theme::Light,
            &renderer::Style {
                text_color: Color::BLACK,
            },
            self.cursor,
        );
        self.cache = Some(ui.into_cache());
    }

    fn frames<'a>(&mut self, view: impl Fn() -> El<'a>, n: usize) {
        for _ in 0..n {
            self.frame(view(), &[]);
        }
    }

    /// Points the cursor at `point` and clicks there.
    fn click_at<'a>(&mut self, view: impl Fn() -> El<'a>, point: Point) {
        self.cursor = mouse::Cursor::Available(point);
        let click: Vec<Event> = iced_test::simulator::click().collect();
        self.frame(view(), &click);
    }
}

fn square<'a>(cache: &TextureCache, colour: Color) -> iced_texture_cache::Cached<'a, Message> {
    cached(cache.clone(), shape().width(50.0).height(50.0).fill(colour))
}

#[test]
fn a_hidden_inner_cache_does_not_keep_its_parent_re_recording() {
    let mut h = Harness::new();
    let outer = TextureCache::new();
    let inner = TextureCache::new();
    let view = || -> El<'_> {
        cached(
            outer.clone(),
            column![square(&inner, Color::BLACK).opacity(0.0), text("visible")],
        )
        .into()
    };
    h.frames(view, 1);
    assert_eq!(outer.record_count(), 1);
    h.frames(view, 3);
    assert_eq!(
        outer.record_count(),
        1,
        "an invisible inner cache must not invalidate its ancestors every frame"
    );
    assert_eq!(inner.record_count(), 0);
}

#[test]
fn an_inner_invalidation_re_records_the_parent_exactly_once() {
    let mut h = Harness::new();
    let outer = TextureCache::new();
    let inner = TextureCache::new();
    let view = || -> El<'_> { cached(outer.clone(), column![square(&inner, Color::BLACK)]).into() };
    h.frames(view, 2);
    assert_eq!((outer.record_count(), inner.record_count()), (1, 1));
    inner.invalidate();
    h.frames(view, 3);
    assert_eq!(
        (outer.record_count(), inner.record_count()),
        (2, 2),
        "one propagation, one re-record each"
    );
}

#[test]
fn a_slide_records_each_nested_cache_once() {
    let mut h = Harness::new();
    let motion = Motion::new();
    let a = TextureCache::new();
    let b = TextureCache::new();
    let view = |current: usize| -> El<'_> {
        motion
            .host(
                pager([
                    El::from(container(square(&a, Color::BLACK)).padding(10)),
                    El::from(container(square(&b, Color::WHITE)).padding(10)),
                ])
                .current(current)
                .motion(motion.clone()),
            )
            .into()
    };
    h.frame(view(0), &[]);
    // The switch: every frame of the slide draws the pager's page textures,
    // not the nested caches.
    for _ in 0..120 {
        h.frame(view(1), &[]);
    }
    assert_eq!(a.record_count(), 1, "page 0's cache was rasterised once");
    assert_eq!(b.record_count(), 1, "page 1's cache was rasterised once");
}

#[test]
fn after_a_settled_backward_switch_the_menu_opens_on_the_shown_page() {
    let mut h = Harness::new();
    let motion = Motion::new();
    // `pick_list` reports no text to operations, so the test works from
    // geometry: the list sits at (0, 100) and is ~31 px tall; its single
    // option opens below it, spanning roughly y ∈ [131, 162].
    let view = |current: usize| -> El<'_> {
        motion
            .host(
                pager([
                    El::from(
                        container(pick_list(["beta"], None::<&'static str>, Message::Picked))
                            .padding(Padding {
                                top: 100.0,
                                ..Padding::ZERO
                            }),
                    ),
                    El::from(text("one")),
                    El::from(text("two")),
                ])
                .current(current)
                .motion(motion.clone()),
            )
            .into()
    };
    h.frame(view(2), &[]);
    for _ in 0..120 {
        h.frame(view(0), &[]);
    }
    // Open the menu on the page that is on screen (x = 0 after the slide).
    h.click_at(|| view(0), Point::new(10.0, 110.0));
    let option = Point::new(10.0, 145.0);
    h.cursor = mouse::Cursor::Available(option);
    h.frame(
        view(0),
        &[Event::Mouse(mouse::Event::CursorMoved { position: option })],
    );
    h.click_at(|| view(0), option);
    assert_eq!(
        h.messages,
        vec![Message::Picked("beta")],
        "the menu belongs to the page on screen, not to a page slid away"
    );
}
