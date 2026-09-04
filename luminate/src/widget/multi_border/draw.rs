//! Ring geometry, kept pure so it can be tested without a renderer.

use iced::advanced::renderer::Quad;
use iced::{Background, Border, Color, Rectangle, Shadow};

use super::{Ring, Side, Style};

/// Every outer ring with the rectangle it occupies, innermost first.
pub(crate) fn outer_rects(
    rings: &[Ring],
    content: Rectangle,
) -> impl Iterator<Item = (Ring, Rectangle)> + '_ {
    let mut cumulative = 0.0;

    rings
        .iter()
        .filter(|ring| ring.side == Side::Outer)
        .map(move |ring| {
            cumulative += ring.offset;
            let expand = cumulative + ring.width;
            cumulative += ring.width;
            (*ring, grow(content, expand))
        })
}

/// The rectangle the background fills and its corner radius.
///
/// With outer rings the fill extends across the innermost outer ring's
/// offset and its corners are concentric with that ring (its radius minus
/// its width and the offset). Without outer rings the fill is the content
/// rectangle, rounded like the first inner ring so its corners never poke
/// out past a rounded ring.
pub(crate) fn background_rect(rings: &[Ring], content: Rectangle) -> (Rectangle, f32) {
    if let Some(ring) = rings.iter().find(|ring| ring.side == Side::Outer) {
        (
            grow(content, ring.offset),
            (ring.radius - ring.width - ring.offset).max(0.0),
        )
    } else {
        let radius = rings
            .iter()
            .find(|ring| ring.side == Side::Inner)
            .map_or(0.0, |ring| ring.radius);
        (content, radius)
    }
}

/// Every inner ring with the rectangle it occupies, outermost first.
pub(crate) fn inner_rects(
    rings: &[Ring],
    content: Rectangle,
) -> impl Iterator<Item = (Ring, Rectangle)> + '_ {
    let mut cumulative = 0.0;

    rings
        .iter()
        .filter(|ring| ring.side == Side::Inner)
        .map(move |ring| {
            cumulative += ring.offset;
            let rect = grow(content, -cumulative);
            cumulative += ring.width;
            (*ring, rect)
        })
}

fn grow(rect: Rectangle, by: f32) -> Rectangle {
    Rectangle {
        x: rect.x - by,
        y: rect.y - by,
        width: (rect.width + 2.0 * by).max(0.0),
        height: (rect.height + 2.0 * by).max(0.0),
    }
}

/// Paints `style` around `content`: background, then the outer rings. The
/// caller draws the content and then [`inner_rings`].
///
/// Rings never overlap (a ring's border sits in the gap its `offset` and
/// `width` reserve, and the quad's fill is transparent), so paint order
/// does not matter and they go down in push order without a buffer.
pub(crate) fn background_and_outer<Renderer: iced::advanced::Renderer>(
    renderer: &mut Renderer,
    style: &Style,
    content: Rectangle,
) {
    if let Some(background) = style.background {
        let (bounds, radius) = background_rect(&style.rings, content);
        renderer.fill_quad(
            Quad {
                bounds,
                border: Border {
                    radius: radius.into(),
                    width: 0.0,
                    color: Color::TRANSPARENT,
                },
                shadow: Shadow::default(),
                snap: false,
            },
            background,
        );
    }

    for (ring, rect) in outer_rects(&style.rings, content) {
        draw_ring(renderer, rect, ring);
    }
}

/// Paints the inner rings over the content.
pub(crate) fn inner_rings<Renderer: iced::advanced::Renderer>(
    renderer: &mut Renderer,
    style: &Style,
    content: Rectangle,
) {
    for (ring, rect) in inner_rects(&style.rings, content) {
        draw_ring(renderer, rect, ring);
    }
}

/// A ring is a transparent quad with a border: iced draws borders inside
/// the quad, so `bounds` is the ring's outer edge.
fn draw_ring<Renderer: iced::advanced::Renderer>(
    renderer: &mut Renderer,
    bounds: Rectangle,
    ring: Ring,
) {
    renderer.fill_quad(
        Quad {
            bounds,
            border: Border {
                radius: ring.radius.into(),
                width: ring.width,
                color: ring.color,
            },
            shadow: Shadow::default(),
            snap: false,
        },
        Background::Color(Color::TRANSPARENT),
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    const CONTENT: Rectangle = Rectangle {
        x: 10.0,
        y: 10.0,
        width: 100.0,
        height: 50.0,
    };

    #[test]
    fn outer_rings_nest_outward_in_push_order() {
        let rings = [
            Ring::outer(2.0, Color::BLACK).offset(1.0),
            Ring::outer(3.0, Color::WHITE).offset(2.0),
        ];
        let rects: Vec<_> = outer_rects(&rings, CONTENT).collect();
        assert_eq!(rects.len(), 2);
        // First ring: 1 gap + 2 width = 3 outside the content.
        assert_eq!(rects[0].1.x, 7.0);
        // Second ring: past the first (3) plus 2 gap plus 3 width = 8.
        assert_eq!(rects[1].1.x, 2.0);
        assert!(rects[1].1.width > rects[0].1.width);
    }

    #[test]
    fn inner_rings_nest_inward_by_width_and_offset() {
        let rings = [
            Ring::inner(1.0, Color::BLACK),
            Ring::inner(2.0, Color::WHITE).offset(3.0),
        ];
        let rects: Vec<_> = inner_rects(&rings, CONTENT).collect();
        assert_eq!(rects.len(), 2);
        // The first ring sits on the content edge.
        assert_eq!(rects[0].1, CONTENT);
        // The second is inset by the first's width (1) plus its own gap (3).
        assert_eq!(rects[1].1.x, 14.0);
        assert_eq!(rects[1].1.width, 92.0);
    }

    #[test]
    fn the_background_stays_concentric_with_the_innermost_outer_ring() {
        let rings = [
            Ring::outer(2.0, Color::BLACK).offset(1.0).radius(6.0),
            Ring::outer(3.0, Color::WHITE).offset(2.0),
        ];
        let (bg, radius) = background_rect(&rings, CONTENT);
        // Extends only across the innermost gap, up to the innermost ring.
        assert_eq!(bg.x, 9.0);
        // Ring radius 6, minus its width 2, minus the 1 px gap = 3.
        assert_eq!(radius, 3.0);
        let innermost_ring = outer_rects(&rings, CONTENT).next().unwrap().1;
        assert!(bg.x >= innermost_ring.x + rings[0].width);
    }

    #[test]
    fn without_outer_rings_the_background_takes_the_first_inner_ring_radius() {
        let rings = [
            Ring::inner(1.0, Color::BLACK).radius(8.0),
            Ring::inner(1.0, Color::WHITE).radius(2.0),
        ];
        assert_eq!(background_rect(&rings, CONTENT), (CONTENT, 8.0));
    }

    #[test]
    fn without_any_ring_the_background_is_the_square_content() {
        assert_eq!(background_rect(&[], CONTENT), (CONTENT, 0.0));
    }

    #[test]
    fn outer_thickness_sums_widths_and_offsets() {
        let style = Style::new()
            .ring(Ring::outer(2.0, Color::BLACK).offset(1.0))
            .ring(Ring::inner(9.0, Color::BLACK))
            .ring(Ring::outer(3.0, Color::WHITE).offset(2.0));
        assert_eq!(style.outer_thickness(), 8.0);
    }
}
