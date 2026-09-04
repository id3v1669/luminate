//! A service locator keyed by marker types, and the [`Shared`] cell it hands
//! out.
//!
//! Pages receive a [`Registry`] in [`Page::new`](crate::Page::new) and use it
//! to share values (a settings struct, a database handle, a draft the user is
//! typing) without the host threading them through.

use std::any::{Any, TypeId};
use std::collections::HashMap;
use std::sync::{Arc, PoisonError, RwLock, RwLockReadGuard, RwLockWriteGuard};

/// Names one slot of a [`Registry`].
///
/// A key is a marker type; the value it addresses is `Key::Value`. Two keys
/// never collide even when they share a value type, so no newtypes are
/// needed and every key can carry documentation.
///
/// ```
/// use iced_page_router::{Key, Registry};
///
/// /// The user's display name.
/// struct DisplayName;
/// impl Key for DisplayName {
///     type Value = String;
/// }
///
/// /// The draft in the search box, another `String`, another slot.
/// struct SearchDraft;
/// impl Key for SearchDraft {
///     type Value = String;
/// }
///
/// let registry = Registry::new();
/// let name = registry.get_or_insert_with::<DisplayName>(|| "ada".to_owned());
/// let draft = registry.get_or_insert_with::<SearchDraft>(String::new);
/// assert_eq!(name.get(), "ada");
/// assert_eq!(draft.get(), "");
/// ```
pub trait Key: 'static {
    /// The value stored under this key.
    ///
    /// `Send + Sync` because a [`Shared`] handle may be moved into a
    /// [`Task`](iced_runtime::Task) future that runs on iced's thread pool.
    type Value: Send + Sync + 'static;
}

type Slot = Box<dyn Any + Send + Sync>;

/// Shared values addressed by [`Key`]. Cloning a `Registry` shares the same
/// slots.
///
/// A [`Router`](crate::Router) owns one and hands it to every page; nested
/// routers usually receive a clone of their parent's registry so pages of
/// both levels see the same values.
#[derive(Debug, Clone, Default)]
pub struct Registry(Arc<RwLock<HashMap<TypeId, Slot>>>);

impl Registry {
    /// Creates an empty registry.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Stores `value` under `K` and returns a handle to it.
    ///
    /// # Errors
    ///
    /// If `K` is already present the existing value is kept and `value` is
    /// handed back untouched, so a caller can recover a connection or a
    /// large buffer.
    ///
    /// ```
    /// use iced_page_router::{Key, Registry};
    ///
    /// struct Port;
    /// impl Key for Port {
    ///     type Value = u16;
    /// }
    ///
    /// let registry = Registry::new();
    /// assert!(registry.insert::<Port>(8080).is_ok());
    /// assert_eq!(registry.insert::<Port>(9090).err(), Some(9090));
    /// assert_eq!(registry.get::<Port>().unwrap().get(), 8080);
    /// ```
    pub fn insert<K: Key>(&self, value: K::Value) -> Result<Shared<K::Value>, K::Value> {
        let mut map = self.0.write().unwrap_or_else(PoisonError::into_inner);

        if map.contains_key(&TypeId::of::<K>()) {
            return Err(value);
        }

        let handle = Shared::new(value);
        let _ = map.insert(TypeId::of::<K>(), Box::new(handle.clone()));

        Ok(handle)
    }

    /// The value under `K`, if any.
    #[must_use]
    pub fn get<K: Key>(&self) -> Option<Shared<K::Value>> {
        self.0
            .read()
            .unwrap_or_else(PoisonError::into_inner)
            .get(&TypeId::of::<K>())
            .and_then(|slot| slot.downcast_ref::<Shared<K::Value>>().cloned())
    }

    /// The value under `K`, inserting `f()` first if the slot is empty.
    ///
    /// `f` runs without the registry lock held, so it may read the registry
    /// itself. If another thread fills the slot meanwhile, that value wins
    /// and `f`'s result is dropped.
    ///
    /// ```
    /// use iced_page_router::{Key, Registry};
    ///
    /// struct Visits;
    /// impl Key for Visits {
    ///     type Value = u32;
    /// }
    ///
    /// let registry = Registry::new();
    /// let visits = registry.get_or_insert_with::<Visits>(|| 0);
    /// visits.update(|n| *n += 1);
    /// assert_eq!(registry.get_or_insert_with::<Visits>(|| 99).get(), 1);
    /// ```
    pub fn get_or_insert_with<K: Key>(&self, f: impl FnOnce() -> K::Value) -> Shared<K::Value> {
        if let Some(existing) = self.get::<K>() {
            return existing;
        }

        let candidate = Shared::new(f());
        let mut map = self.0.write().unwrap_or_else(PoisonError::into_inner);

        map.entry(TypeId::of::<K>())
            .or_insert_with(|| Box::new(candidate.clone()))
            .downcast_ref::<Shared<K::Value>>()
            .cloned()
            .unwrap_or_else(|| {
                unreachable!("a registry slot for `K` always holds `Shared<K::Value>`")
            })
    }
}

/// A cheaply cloneable, lock-protected shared value.
///
/// # Why `Arc<RwLock<T>>` and not `Rc<RefCell<T>>`
///
/// iced's `update` runs on one thread, but a handle is routinely moved into
/// a [`Task`](iced_runtime::Task) future, and with iced's `thread-pool`
/// executor that future may run on another thread. `Shared<T>` is therefore
/// `Send + Sync` exactly when `T: Send + Sync`, and the [`Registry`] slots
/// require the same. A poisoned lock is recovered
/// ([`PoisonError::into_inner`]), never propagated.
///
/// Locks are not re-entrant: holding a guard while constructing a page that
/// locks the same value deadlocks. Prefer the guard-free helpers
/// ([`get`](Self::get), [`set`](Self::set), [`with`](Self::with),
/// [`update`](Self::update)) and never hold a guard across a call into the
/// router.
///
/// ```
/// use iced_page_router::Shared;
///
/// let a: Shared<Vec<u8>> = Shared::new(vec![1]);
/// let b = a.clone();
/// b.update(|v| v.push(2));
/// assert_eq!(a.with(Vec::len), 2);
/// assert!(a.ptr_eq(&b));
/// assert!(!a.ptr_eq(&Shared::from(vec![1, 2])));
/// ```
#[derive(Debug)]
pub struct Shared<T>(Arc<RwLock<T>>);

impl<T> Shared<T> {
    /// Wraps `value`.
    #[must_use]
    pub fn new(value: T) -> Self {
        Self(Arc::new(RwLock::new(value)))
    }

    /// Read access. Keep the guard short.
    pub fn read(&self) -> RwLockReadGuard<'_, T> {
        self.0.read().unwrap_or_else(PoisonError::into_inner)
    }

    /// Write access. Keep the guard short.
    pub fn write(&self) -> RwLockWriteGuard<'_, T> {
        self.0.write().unwrap_or_else(PoisonError::into_inner)
    }

    /// Replaces the value.
    pub fn set(&self, value: T) {
        *self.write() = value;
    }

    /// A copy of the value.
    #[must_use]
    pub fn get(&self) -> T
    where
        T: Clone,
    {
        self.read().clone()
    }

    /// Runs `f` on a shared borrow of the value.
    pub fn with<R>(&self, f: impl FnOnce(&T) -> R) -> R {
        f(&self.read())
    }

    /// Runs `f` on an exclusive borrow of the value.
    pub fn update<R>(&self, f: impl FnOnce(&mut T) -> R) -> R {
        f(&mut self.write())
    }

    /// Whether both handles point at the same value.
    #[must_use]
    pub fn ptr_eq(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.0, &other.0)
    }
}

impl<T> Clone for Shared<T> {
    fn clone(&self) -> Self {
        Self(Arc::clone(&self.0))
    }
}

impl<T: Default> Default for Shared<T> {
    fn default() -> Self {
        Self::new(T::default())
    }
}

impl<T> From<T> for Shared<T> {
    fn from(value: T) -> Self {
        Self::new(value)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct Count;
    impl Key for Count {
        type Value = u32;
    }

    struct OtherCount;
    impl Key for OtherCount {
        type Value = u32;
    }

    struct Name;
    impl Key for Name {
        type Value = String;
    }

    #[test]
    fn insert_twice_keeps_the_first_and_returns_the_second_value() {
        let r = Registry::new();
        let first = r.insert::<Count>(1).unwrap();
        assert_eq!(r.insert::<Count>(2).err(), Some(2));
        assert_eq!(first.get(), 1);
        assert_eq!(r.get::<Count>().unwrap().get(), 1);
    }

    #[test]
    fn keys_with_the_same_value_type_do_not_collide() {
        let r = Registry::new();
        let _ = r.insert::<Count>(1).unwrap();
        let _ = r.insert::<OtherCount>(2).unwrap();
        assert_eq!(r.get::<Count>().unwrap().get(), 1);
        assert_eq!(r.get::<OtherCount>().unwrap().get(), 2);
    }

    #[test]
    fn get_on_a_missing_key_is_none() {
        assert!(Registry::new().get::<Name>().is_none());
    }

    #[test]
    fn get_or_insert_with_may_use_the_registry() {
        let r = Registry::new();
        let _ = r.insert::<Name>("cfg".to_owned()).unwrap();
        let derived =
            r.get_or_insert_with::<Count>(|| r.get::<Name>().unwrap().with(|s| s.len() as u32));
        assert_eq!(derived.get(), 3);
        assert_eq!(r.get_or_insert_with::<Count>(|| 99).get(), 3);
    }

    #[test]
    fn a_cloned_registry_shares_slots() {
        let a = Registry::new();
        let b = a.clone();
        let _ = a.insert::<Count>(7).unwrap();
        assert_eq!(b.get::<Count>().unwrap().get(), 7);
    }

    #[test]
    fn shared_helpers_round_trip() {
        let s = Shared::from(String::from("a"));
        s.set("b".to_owned());
        s.update(|v| v.push('c'));
        assert_eq!(s.with(String::len), 2);
        assert_eq!(s.get(), "bc");
        assert!(s.ptr_eq(&s.clone()));
        assert!(!s.ptr_eq(&Shared::default()));
        assert!(format!("{s:?}").contains("Shared"));
    }
}
