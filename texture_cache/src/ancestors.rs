//! The ancestor protocol: how an inner texture's invalidation reaches the
//! outer textures it is baked into.
//!
//! A widget that records a texture wraps its content's `update` in
//! [`with_ancestor`]. Any texture-recording widget *inside* that content
//! then calls [`propagate`] after its own `update`: if its cache was
//! invalidated since the last time it propagated, every ancestor on the
//! stack is invalidated too, so the outer texture re-records in the same
//! frame. Propagation is keyed on [`TextureCache::generation`], not on the
//! invalidated *flag*: a cache that is invalidated but never drawn (hidden,
//! off-screen, fully transparent) keeps its flag set, and would otherwise
//! re-invalidate its ancestors on every event for as long as it is hidden.
//!
//! The stack is thread-local because iced's widget tree is updated on one
//! thread, re-entrantly; a third widget that records textures must follow
//! the same two calls.

use std::cell::RefCell;

use crate::texture_cache::TextureCache;

thread_local! {
    /// The caches whose content `update` is currently on the stack,
    /// outermost first.
    static ANCESTORS: RefCell<Vec<TextureCache>> = const { RefCell::new(Vec::new()) };
}

/// Runs `f` with `cache` registered as an ancestor of everything `f`
/// updates.
pub(crate) fn with_ancestor<R>(cache: &TextureCache, f: impl FnOnce() -> R) -> R {
    /// Pops on the way out even if `f` unwinds, so a panic caught further up
    /// cannot leave a dead ancestor on the stack.
    struct Pop;

    impl Drop for Pop {
        fn drop(&mut self) {
            ANCESTORS.with(|stack| {
                let _ = stack.borrow_mut().pop();
            });
        }
    }

    ANCESTORS.with(|stack| stack.borrow_mut().push(cache.clone()));
    let _guard = Pop;
    f()
}

/// Invalidates every registered ancestor unconditionally. For a widget whose
/// own image changes every frame without any cache being invalidated (a
/// `Pager` mid-slide composites its page textures at a moving offset).
pub(crate) fn invalidate_ancestors() {
    ANCESTORS.with(|stack| {
        for ancestor in stack.borrow().iter() {
            ancestor.invalidate();
        }
    });
}

/// Invalidates every registered ancestor if `cache` was invalidated since
/// the generation stored in `propagated`, and records the new generation.
/// Returns whether it propagated. Call it *after* the content's `update`
/// (outside the widget's own [`with_ancestor`] scope).
pub(crate) fn propagate(cache: &TextureCache, propagated: &mut u64) -> bool {
    let generation = cache.generation();
    if generation == *propagated {
        return false;
    }
    *propagated = generation;
    invalidate_ancestors();
    true
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn propagation_happens_once_per_invalidation() {
        let outer = TextureCache::new();
        let inner = TextureCache::new();
        let _ = outer.take_invalidated();
        let mut slot = 0;

        // A fresh cache counts as invalidated once.
        with_ancestor(&outer, || assert!(propagate(&inner, &mut slot)));
        assert!(outer.take_invalidated());

        // Nothing changed: the still-set flag of `inner` is not propagated again.
        assert!(inner.is_invalidated());
        with_ancestor(&outer, || assert!(!propagate(&inner, &mut slot)));
        assert!(!outer.is_invalidated());

        // An explicit invalidation reaches the outer exactly once more.
        inner.invalidate();
        with_ancestor(&outer, || assert!(propagate(&inner, &mut slot)));
        assert!(outer.take_invalidated());
        with_ancestor(&outer, || assert!(!propagate(&inner, &mut slot)));
        assert!(!outer.is_invalidated());
    }

    #[test]
    fn every_ancestor_on_the_stack_is_invalidated() {
        let grand = TextureCache::new();
        let parent = TextureCache::new();
        let child = TextureCache::new();
        let _ = (grand.take_invalidated(), parent.take_invalidated());
        let mut slot = 0;

        with_ancestor(&grand, || {
            with_ancestor(&parent, || {
                assert!(propagate(&child, &mut slot));
            });
        });
        assert!(grand.is_invalidated() && parent.is_invalidated());
    }

    #[test]
    fn invalidate_ancestors_reaches_every_registered_cache() {
        let outer = TextureCache::new();
        let _ = outer.take_invalidated();
        with_ancestor(&outer, invalidate_ancestors);
        assert!(outer.is_invalidated());
    }

    #[test]
    fn the_stack_is_empty_outside_with_ancestor() {
        let outer = TextureCache::new();
        let _ = outer.take_invalidated();
        with_ancestor(&outer, || {});
        let mut slot = 0;
        assert!(propagate(&TextureCache::new(), &mut slot));
        assert!(
            !outer.is_invalidated(),
            "nothing registered after the scope ended"
        );
    }

    #[test]
    fn a_panic_inside_the_scope_still_pops() {
        let outer = TextureCache::new();
        let _ = outer.take_invalidated();
        let result = std::panic::catch_unwind(|| {
            with_ancestor(&outer, || panic!("boom"));
        });
        assert!(result.is_err());
        let mut slot = 0;
        assert!(propagate(&TextureCache::new(), &mut slot));
        assert!(!outer.is_invalidated(), "the dead ancestor was popped");
    }
}
