use iced::{Background, Color};

/// Which side of the content a ring sits on.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Side {
    /// Outside the content, growing the widget.
    Outer,
    /// Inside the content's edge, drawn on top of it.
    Inner,
}

/// One ring of border.
///
/// Rings on the same side are ordered by distance from the content edge:
/// the first pushed touches the edge, the next sits beyond it, and so on.
/// outward for [`Side::Outer`], inward for [`Side::Inner`].
/// [`offset`](Self::offset) is the gap between a ring and the previous one
/// on its side (or the content edge for the first).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Ring {
    /// Whether the ring sits outside or inside the content.
    pub side: Side,
    /// Thickness of the ring.
    pub width: f32,
    /// Colour of the ring.
    pub color: Color,
    /// Corner radius of the ring.
    pub radius: f32,
    /// Gap toward the content edge: to the previous ring on this side, or
    /// to the content for the first ring.
    pub offset: f32,
}

impl Ring {
    /// A ring outside the content.
    ///
    /// # Panics
    /// In debug builds, when `width` is negative or not finite.
    #[must_use]
    pub fn outer(width: f32, color: Color) -> Self {
        Self::new(Side::Outer, width, color)
    }

    /// A ring just inside the content's edge.
    ///
    /// # Panics
    /// In debug builds, when `width` is negative or not finite.
    #[must_use]
    pub fn inner(width: f32, color: Color) -> Self {
        Self::new(Side::Inner, width, color)
    }

    fn new(side: Side, width: f32, color: Color) -> Self {
        debug_assert!(
            width.is_finite() && width >= 0.0,
            "ring width must be finite and non-negative, got {width}"
        );

        Self {
            side,
            width,
            color,
            radius: 0.0,
            offset: 0.0,
        }
    }

    /// Sets the corner radius.
    ///
    /// # Panics
    /// In debug builds, when `radius` is negative or not finite.
    #[must_use]
    pub fn radius(mut self, radius: f32) -> Self {
        debug_assert!(
            radius.is_finite() && radius >= 0.0,
            "ring radius must be finite and non-negative, got {radius}"
        );
        self.radius = radius;
        self
    }

    /// Sets the gap toward the content edge. Rings never overlap, so the
    /// gap cannot be negative.
    ///
    /// # Panics
    /// In debug builds, when `offset` is negative or not finite.
    #[must_use]
    pub fn offset(mut self, offset: f32) -> Self {
        debug_assert!(
            offset.is_finite() && offset >= 0.0,
            "ring offset must be finite and non-negative, got {offset}"
        );
        self.offset = offset;
        self
    }
}

/// The interaction state a [`Style`] is computed for.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[non_exhaustive]
pub struct Status {
    /// Pointer is pressed on the content.
    pub is_pressed: bool,
    /// Pointer is over the content.
    pub is_hovered: bool,
    /// The widget is focused.
    pub is_focused: bool,
    /// The widget is disabled.
    pub is_disabled: bool,
}

/// What a [`MultiBorder`](super::MultiBorder) draws.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct Style {
    /// Rings, each side ordered from the content edge outward (see
    /// [`Ring`]).
    pub rings: Vec<Ring>,
    /// Fill behind the content. It reaches the innermost outer ring and is
    /// rounded to stay concentric with it; without outer rings it is the
    /// content rectangle rounded like the first inner ring.
    pub background: Option<Background>,
}

impl Style {
    /// An empty style: no rings, no background.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Adds a ring.
    #[must_use]
    pub fn ring(mut self, ring: Ring) -> Self {
        self.rings.push(ring);
        self
    }

    /// Sets the background fill.
    #[must_use]
    pub fn background(mut self, background: impl Into<Background>) -> Self {
        self.background = Some(background.into());
        self
    }

    /// Total thickness of the outer rings (widths plus offsets): what the
    /// layout must reserve around the content.
    #[must_use]
    pub fn outer_thickness(&self) -> f32 {
        self.rings
            .iter()
            .filter(|ring| ring.side == Side::Outer)
            .map(|ring| ring.width + ring.offset)
            .sum()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn styles_compare_by_value() {
        let a = Style::new().ring(Ring::outer(1.0, Color::BLACK).radius(2.0));
        let b = Style::new().ring(Ring::outer(1.0, Color::BLACK).radius(2.0));
        assert_eq!(a, b);
        assert_ne!(a, Style::new());
    }

    #[test]
    #[should_panic(expected = "ring offset must be finite and non-negative")]
    fn a_negative_offset_is_a_programming_error() {
        let _ = Ring::inner(1.0, Color::BLACK).offset(-1.0);
    }

    #[test]
    #[should_panic(expected = "ring width must be finite and non-negative")]
    fn a_negative_width_is_a_programming_error() {
        let _ = Ring::outer(-1.0, Color::BLACK);
    }
}
