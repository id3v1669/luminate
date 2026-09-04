//! Named property sets are the main way to declare an animation.
//!
//! A transition in a real interface is rarely one property. Showing a progress
//! row grows the label, opens the bar, and adds vertical padding; those are
//! three properties but *one* state change. Declaring them one call at a time
//! makes the call site grow with the animation and scatters a single idea
//! across the view.
//!
//! A [`MotionSet`] is that idea as a value: a plain struct of target values,
//! declared once as a constant, per state. The view names a state; the engine
//! animates every field toward it under one curve.
//!
//! ```
//! use iced::Padding;
//! use iced_animate::{curves::SMOOTH, key, motion_set, Motion};
//!
//! motion_set! {
//!     /// Geometry of one progress row.
//!     pub struct ItemStyle -> ItemStyleAnim {
//!         name_size: f32,
//!         item_pad: Padding,
//!         bar_height: f32,
//!     }
//! }
//!
//! const COLLAPSED: ItemStyle = ItemStyle { name_size: 14.0, item_pad: Padding::ZERO, bar_height: 0.0 };
//! const EXPANDED: ItemStyle = ItemStyle { name_size: 18.0, item_pad: Padding::new(8.0), bar_height: 6.0 };
//!
//! let m = Motion::new();
//! let shown = true;
//! let s: ItemStyleAnim = m.to_set(key!(), SMOOTH, if shown { EXPANDED } else { COLLAPSED });
//! assert_eq!(s.bar_height.get(), 6.0);
//! ```
//!
//! `s` is the generated twin of `ItemStyle` with every field wrapped in
//! [`Anim`], ready to hand to widgets that read their values in `layout`
//! and `draw`. The macro derives `Debug, Clone, Copy, PartialEq` on the
//! source struct; do not add those derives yourself.
//!
//! [`Anim`]: crate::Anim

use crate::{Curve, Motion, MotionKey};

/// A struct of target values that animate together under one curve.
///
/// Implemented by the [`motion_set!`] macro; there is little reason to write
/// it by hand.
///
/// [`motion_set!`]: crate::motion_set
pub trait MotionSet: Copy + 'static {
    /// The same struct with every field wrapped in [`Anim`].
    ///
    /// [`Anim`]: crate::Anim
    type Animated;

    /// Registers a track per field and returns the animated twin.
    ///
    /// Each field derives its own key from `key` via [`MotionKey::salted`],
    /// so the fields stay independent tracks while sharing one identity.
    /// Called by [`Motion::to_set`]; prefer that.
    fn bind(self, motion: &Motion, key: MotionKey, curve: Curve) -> Self::Animated;
}

/// Declares a property set, its animated twin, and the wiring between them.
///
/// The twin is named explicitly rather than derived, so the generated type is
/// greppable and the macro needs no identifier-concatenation dependency.
///
/// ```
/// use iced::Padding;
/// use iced_animate::motion_set;
///
/// motion_set! {
///     /// Geometry of one progress row.
///     pub struct ItemStyle -> ItemStyleAnim {
///         /// Text size of the name.
///         name_size: f32,
///         /// Padding around the row.
///         item_pad: Padding,
///     }
/// }
/// ```
///
/// The macro adds `#[derive(Debug, Clone, Copy, PartialEq)]` to the source
/// struct; the twin derives `Debug, Clone` and implements `Default`, so
/// **every field type must implement `Default`** (all of iced's animatable
/// types do). Field attributes, including doc comments, are forwarded to
/// the twin's fields, so a `pub` set documents its twin too.
///
/// [`Anim`]: crate::Anim
#[macro_export]
macro_rules! motion_set {
    (
        $(#[$meta:meta])*
        $vis:vis struct $name:ident -> $animated:ident {
            $(
                $(#[$field_meta:meta])*
                $field:ident : $ty:ty
            ),* $(,)?
        }
    ) => {
        $(#[$meta])*
        #[derive(Debug, Clone, Copy, PartialEq)]
        $vis struct $name {
            $(
                $(#[$field_meta])*
                pub $field: $ty,
            )*
        }

        #[doc = concat!(
            "Animated twin of [`", stringify!($name), "`]: the same fields, \
             each resolved on read rather than at view-build time."
        )]
        #[derive(Debug, Clone)]
        $vis struct $animated {
            $(
                $(#[$field_meta])*
                pub $field: $crate::Anim<$ty>,
            )*
        }

        impl ::core::default::Default for $animated {
            fn default() -> Self {
                Self {
                    $($field: ::core::default::Default::default(),)*
                }
            }
        }

        impl $crate::MotionSet for $name {
            type Animated = $animated;

            fn bind(
                self,
                motion: &$crate::Motion,
                key: $crate::MotionKey,
                curve: $crate::Curve,
            ) -> Self::Animated {
                $animated {
                    $(
                        $field: motion.to(
                            key.salted(const {
                                $crate::__private::site_hash(
                                    ::core::stringify!($field),
                                    0,
                                    0,
                                )
                            }),
                            curve,
                            self.$field,
                        ),
                    )*
                }
            }
        }
    };
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use iced::Padding;

    use crate::testing::FrameClock;
    use crate::{Curve, Motion, SpringParams, key};

    /// A fast spring for tests; not the shipped `curves::SMOOTH`.
    const FAST: Curve = Curve::spring(SpringParams::new(0.0, Duration::from_millis(300)));

    crate::motion_set! {
        /// Fixture for the [`crate::motion_set!`] expansion.
        struct RowStyle -> RowStyleAnim {
            /// Row height in logical pixels.
            height: f32,
            /// Inner padding.
            pad: Padding,
        }
    }

    #[test]
    fn a_motion_set_twin_defaults_to_its_field_defaults() {
        let twin = RowStyleAnim::default();
        assert_eq!(twin.height.get(), 0.0);
        assert_eq!(twin.pad.get(), Padding::ZERO);
        assert!(!twin.height.is_live());
    }

    #[test]
    fn a_motion_set_animates_every_field_under_one_curve() {
        const COLLAPSED: RowStyle = RowStyle {
            height: 0.0,
            pad: Padding {
                top: 0.0,
                right: 0.0,
                bottom: 0.0,
                left: 0.0,
            },
        };

        const EXPANDED: RowStyle = RowStyle {
            height: 48.0,
            pad: Padding {
                top: 8.0,
                right: 12.0,
                bottom: 8.0,
                left: 12.0,
            },
        };

        let m = Motion::new();
        let mut clock = FrameClock::new(&m);
        let key = key!();

        let _ = m.to_set(key, FAST, COLLAPSED);
        let s = m.to_set(key, FAST, EXPANDED);

        assert_eq!(m.track_count(), 2, "one track per field, not per component");
        assert!(s.height.is_animating());
        assert!(s.pad.is_animating());

        let _ = clock.run(120);

        assert_eq!(s.height.get(), 48.0);
        assert_eq!(s.pad.get().left, 12.0);
        assert_eq!(s.pad.get().top, 8.0);
    }

    #[test]
    fn set_fields_keep_independent_identities() {
        let m = Motion::new();

        // Two different keys over the same set must not share tracks.
        let _ = m.to_set(
            key!("a"),
            FAST,
            RowStyle {
                height: 1.0,
                pad: Padding::ZERO,
            },
        );
        let _ = m.to_set(
            key!("b"),
            FAST,
            RowStyle {
                height: 2.0,
                pad: Padding::ZERO,
            },
        );

        assert_eq!(m.track_count(), 4);
    }
}
