//! Metric helpers the tokens need as `const` (iced's own constructors are
//! not) and the one ring-geometry rule. Crate-private on purpose: a theme
//! author writes the resulting values into the token structs directly.

use iced::Padding;

/// Symmetric vertical/horizontal padding.
pub(crate) const fn padding_vh(vertical: f32, horizontal: f32) -> Padding {
    Padding {
        top: vertical,
        right: horizontal,
        bottom: vertical,
        left: horizontal,
    }
}

/// Corner radius of an outer ring drawn `offset` outside a control whose own
/// corner radius is `radius`, so the ring stays concentric with the corner.
/// Used by the button's pressed ring and the input's focus ring.
pub(crate) const fn ring_radius(radius: f32, offset: f32, width: f32) -> f32 {
    radius + offset + width
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ring_radius_is_concentric() {
        assert_eq!(ring_radius(10.0, 2.0, 2.0), 14.0);
        assert_eq!(ring_radius(10.0, 0.0, 3.5), 13.5);
    }

    #[test]
    fn padding_vh_is_symmetric() {
        let p = padding_vh(7.0, 15.0);
        assert_eq!((p.top, p.bottom, p.left, p.right), (7.0, 7.0, 15.0, 15.0));
    }
}
