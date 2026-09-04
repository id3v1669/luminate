//! Stable identities for animation tracks.
//!
//! iced rebuilds the widget tree on every `view()`, so an animation cannot be
//! addressed by tree position, the position is not stable and does not
//! survive a reorder. A [`MotionKey`] is the address instead: it combines the
//! *call site* that produced it with an optional runtime discriminator, so two
//! list items built by the same line of code get distinct keys while the same
//! item keeps its key across rebuilds.
//!
//! Build keys with the [`key!`] macro rather than by hand.
//!
//! [`key!`]: crate::key

use std::hash::{DefaultHasher, Hash, Hasher};
use std::sync::atomic::{AtomicU64, Ordering};

const FNV_OFFSET: u64 = 0xcbf2_9ce4_8422_2325;
const FNV_PRIME: u64 = 0x0000_0100_0000_01b3;

/// Hashes a source location into a call-site salt.
///
/// Evaluated in a `const` block by [`key!`], so the per-frame cost of building
/// a key is the runtime discriminator alone.
///
/// [`key!`]: crate::key
#[doc(hidden)]
#[must_use]
pub const fn site_hash(module: &str, line: u32, column: u32) -> u64 {
    let bytes = module.as_bytes();
    let mut hash = FNV_OFFSET;
    let mut i = 0;

    while i < bytes.len() {
        hash ^= bytes[i] as u64;
        hash = hash.wrapping_mul(FNV_PRIME);
        i += 1;
    }

    hash ^= line as u64;
    hash = hash.wrapping_mul(FNV_PRIME);
    hash ^= column as u64;
    hash.wrapping_mul(FNV_PRIME)
}

/// The address of an animation track within a [`Motion`] engine.
///
/// [`Motion`]: crate::Motion
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct MotionKey(u64);

impl MotionKey {
    /// Wraps a precomputed hash. Prefer the [`key!`] macro.
    ///
    /// [`key!`]: crate::key
    #[must_use]
    pub const fn from_raw(raw: u64) -> Self {
        Self(raw)
    }

    /// Returns the raw hash behind this key.
    #[must_use]
    pub const fn raw(self) -> u64 {
        self.0
    }

    /// Mixes a runtime discriminator into the key.
    ///
    /// This is what separates one list item from another when both are built
    /// by the same line of code. `D` is taken by reference and hashed with
    /// the standard hasher, so a `&String` and a `&str` with the same text
    /// produce the same key.
    #[must_use]
    pub fn with<D: Hash + ?Sized>(self, discriminator: &D) -> Self {
        let mut hasher = DefaultHasher::new();
        self.0.hash(&mut hasher);
        discriminator.hash(&mut hasher);
        Self(hasher.finish())
    }

    /// Allocates a process-unique key for animation state that belongs to the
    /// widget tree rather than to the application.
    ///
    /// Most keys come from [`key!`], which ties identity to a call site and a
    /// piece of application data. Some animations have no such identity: a
    /// sidebar's collapse, a pager's slide. Their state is born and dies with
    /// the widget. `tree::State` persists across view rebuilds, is dropped
    /// with the widget, and already expresses that
    /// lifetime.
    ///
    /// Such a widget allocates one of these when its `tree::State` is created
    /// and keeps it there. The key is then as stable as the widget, and once
    /// the widget goes the key stops being touched and the engine's garbage
    /// collector reclaims the track.
    ///
    /// [`key!`]: crate::key
    #[must_use]
    pub fn unique() -> Self {
        static NEXT: AtomicU64 = AtomicU64::new(1);

        // Spread the counter across the key space so a collision with a
        // hashed key is as unlikely as one between two hashed keys.
        let ordinal = NEXT.fetch_add(1, Ordering::Relaxed);

        Self((ordinal ^ FNV_OFFSET).wrapping_mul(FNV_PRIME))
    }

    /// Derives a stable sub-key by mixing in a compile-time salt.
    ///
    /// Each field of a [`MotionSet`] gets its own track, keyed by a
    /// `const`-hashed field name. Naming rather than numbering the fields
    /// keeps every other field's key stable when one is renamed or reordered.
    ///
    /// [`MotionSet`]: crate::MotionSet
    #[must_use]
    pub const fn salted(self, salt: u64) -> Self {
        Self((self.0 ^ salt).wrapping_mul(FNV_PRIME))
    }
}

/// Builds a [`MotionKey`] from the call site plus optional discriminators.
///
/// ```
/// use iced_animate::key;
/// struct Stage { id: u32, name: String }
/// let stage = Stage { id: 7, name: "seven".into() };
/// let a = key!();                    // unique to this line
/// let b = key!(stage.id);            // one track per stage
/// let c = key!(stage.id, "opacity"); // several tracks per stage
/// let d = key!(stage.name);          // taken by reference: `name` is not moved
/// assert_ne!(a, b);
/// assert_ne!(b, c);
/// assert_ne!(c, d);
/// assert_eq!(stage.name, "seven");
/// ```
///
/// Discriminators are hashed **by reference**, so a `String` field of a
/// borrowed item works and nothing is moved.
///
/// # Pitfalls
///
/// The call site is the *outermost* macro invocation: a `key!()` expanded
/// inside your own `macro_rules!`, or in a closure called from a loop, yields
/// the same key every time. Mix in runtime data (`key!(item.id)`) whenever
/// one line of code builds more than one animated thing.
///
/// A generic function is one call site for every instantiation:
/// `fn row<T>() { key!() }` produces the same key for `row::<A>()` and
/// `row::<B>()`. Discriminate by type when that matters:
/// `key!(std::any::TypeId::of::<T>())`.
///
/// [`MotionKey`]: crate::MotionKey
//
// The built-ins below are written as absolute paths on purpose. `column!` is
// resolved in the *caller's* scope, and virtually every iced view starts with
// `use iced::widget::column`, which shadows it, the macro would then hash a
// `Column` widget instead of a source column and fail to compile.
#[macro_export]
macro_rules! key {
    () => {
        $crate::MotionKey::from_raw(const {
            $crate::__private::site_hash(
                ::core::module_path!(),
                ::core::line!(),
                ::core::column!(),
            )
        })
    };
    ($($part:expr),+ $(,)?) => {
        $crate::MotionKey::from_raw(const {
            $crate::__private::site_hash(
                ::core::module_path!(),
                ::core::line!(),
                ::core::column!(),
            )
        })
        .with(&($(&$part,)+))
    };
}

#[cfg(test)]
mod tests {
    use std::any::TypeId;

    use crate::MotionKey;
    use std::time::Duration;

    use crate::testing::FrameClock;
    use crate::{Curve, Motion, SpringParams};

    struct Item {
        name: String,
    }

    #[test]
    fn key_takes_a_non_copy_discriminator_by_reference() {
        // Must compile without moving `name` out of `item`. One site, called
        // twice, as a rebuild does: two `key!` lines would be two sites.
        fn site(by_ref: &Item) -> MotionKey {
            crate::key!(by_ref.name)
        }

        let item = Item {
            name: String::from("alpha"),
        };
        let a = site(&item);
        let b = site(&item);
        assert_eq!(a, b);
        assert_eq!(
            item.name, "alpha",
            "the discriminator is still owned by the item"
        );
    }

    #[test]
    fn a_reference_hashes_like_the_value() {
        let site = MotionKey::from_raw(1);
        let value = site.with(&7_u32);
        let reference = site.with(&&7_u32);
        assert_eq!(value, reference);
    }

    #[test]
    fn raw_round_trips() {
        assert_eq!(MotionKey::from_raw(42).raw(), 42);
    }

    #[test]
    fn salted_keys_are_stable_and_distinct() {
        let base = MotionKey::from_raw(9);
        assert_eq!(base.salted(7), base.salted(7));
        assert_ne!(base.salted(7), base.salted(8));
        assert_ne!(base.salted(7), base);
    }

    #[test]
    fn a_generic_function_shares_one_site_across_instantiations() {
        // The unused `T` is the point: one site, several instantiations.
        #[allow(clippy::extra_unused_type_parameters)]
        fn colliding<T: 'static>() -> MotionKey {
            crate::key!()
        }
        fn separated<T: 'static>() -> MotionKey {
            crate::key!(TypeId::of::<T>())
        }

        assert_eq!(
            colliding::<u8>(),
            colliding::<u16>(),
            "one call site is one key, whatever `T` is"
        );
        assert_ne!(separated::<u8>(), separated::<u16>());
    }

    #[test]
    fn key_accepts_any_expression() {
        let a = crate::key!(const { 1_u8 });
        let b = crate::key!(const { 2_u8 });
        assert_ne!(a, b);
    }

    /// A fast spring for tests; not the shipped `curves::SMOOTH`.
    const FAST: Curve = Curve::spring(SpringParams::new(0.0, Duration::from_millis(300)));

    #[test]
    fn keys_are_stable_per_site_and_distinct_per_discriminator() {
        // Each `key!()` expansion is its own site, so stability has to be observed
        // by calling one site repeatedly. A rebuild does exactly that.
        fn site() -> crate::MotionKey {
            key!()
        }

        fn row(id: u32) -> crate::MotionKey {
            key!("row", id)
        }

        let first = key!();
        let second = key!();

        assert_ne!(first, second, "two call sites must not collide");
        assert_eq!(site(), site(), "one site must be stable across calls");
        assert_ne!(site(), first, "and distinct from another site");

        assert_ne!(row(1), row(2), "discriminators must separate keys");
        assert_eq!(row(1), row(1), "the same discriminator must be stable");
    }

    #[test]
    fn key_identity_survives_rebuilds() {
        fn key_for(id: u32) -> crate::MotionKey {
            key!(id)
        }

        assert_eq!(key_for(7), key_for(7));
        assert_ne!(key_for(7), key_for(8));
    }

    #[test]
    fn unique_keys_are_distinct() {
        let a = crate::MotionKey::unique();
        let b = crate::MotionKey::unique();

        assert_ne!(a, b, "each widget instance must get its own identity");
        assert_ne!(a, key!(), "and never collides with a call site");
    }

    #[test]
    fn a_unique_key_addresses_a_normal_track() {
        let m = Motion::new();
        let mut clock = FrameClock::new(&m);
        let key = crate::MotionKey::unique();

        let _ = m.to(key, FAST, 0.0_f32);
        let value = m.to(key, FAST, 100.0_f32);

        let _ = clock.run(120);

        assert_eq!(value.get(), 100.0);
    }

    #[test]
    fn the_key_macro_survives_a_shadowed_column() {
        // Nearly every iced view starts with this import, and it shadows the
        // built-in `column!` that `key!` expands to. The macro spells the
        // built-in as `::core::column!` so this compiles.
        use iced::widget::column;

        fn site() -> crate::MotionKey {
            use iced::widget::column;

            let _widget: iced::widget::Column<'_, ()> = column![];

            key!("row", 1_u32)
        }

        let _widget: iced::widget::Column<'_, ()> = column![];

        assert_ne!(key!(), key!());
        assert_eq!(site(), site());
    }
}
