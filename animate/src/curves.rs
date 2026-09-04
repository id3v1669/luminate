//! Shared animation curve presets.
//!
//! Declared in one place so that everything built on the engine moves with
//! one voice, and retuning the feel of an interface is an edit in one file.

use std::time::Duration;

use crate::Easing;

use crate::{Curve, SpringParams};

/// Parameters behind [`SMOOTH`]; also [`SpringParams::default`].
pub(crate) const SMOOTH_PARAMS: SpringParams = SpringParams::new(0.0, Duration::from_millis(400));

/// The default transition: settles quickly, does not overshoot.
///
/// Use this unless there is a reason not to. It is a spring, so a value
/// retargeted mid-flight continues from its current motion.
pub const SMOOTH: Curve = Curve::spring(SMOOTH_PARAMS);

/// A brisker [`SMOOTH`], for small elements that should feel immediate.
pub const QUICK: Curve = Curve::spring(SpringParams::new(0.0, Duration::from_millis(220)));

/// Overshoots slightly on arrival, for something appearing, not adjusting.
pub const BOUNCY: Curve = Curve::spring(SpringParams::new(0.35, Duration::from_millis(500)));

/// Structural motion: a panel collapsing, a stack sliding between pages.
///
/// Slower and heavier than [`SMOOTH`] because it moves a large area, where a
/// fast transition reads as a jump rather than a movement.
pub const STRUCTURAL: Curve = Curve::spring(SpringParams::new(0.0, Duration::from_millis(450)));

/// How long [`FADE`] takes.
///
/// Named so [`COLLAPSE`] can wait exactly that long rather than repeating a
/// number that would drift out of step the first time the fade is retuned.
const FADE_MS: u64 = 180;

/// Closing the space an element leaves behind, once it has faded out.
///
/// Delayed by the full length of [`FADE`], on purpose. A collapse clips what
/// it shrinks, so overlapping the two cuts the element in half against
/// whatever lies below while it is still visible. Waiting means the clip only
/// ever eats something already invisible.
pub const COLLAPSE: Curve = Curve::ease(Easing::EaseInOut, Duration::from_millis(220))
    .delayed(Duration::from_millis(FADE_MS));

/// A plain fade, where spring physics would be meaningless.
///
/// Opacity has no momentum to preserve, so a duration curve is both cheaper
/// and more predictable than a spring.
pub const FADE: Curve = Curve::ease(Easing::EaseOut, Duration::from_millis(FADE_MS));
