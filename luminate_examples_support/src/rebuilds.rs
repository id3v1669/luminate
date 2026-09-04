//! Proof that animating rebuilds nothing.

use std::cell::Cell;

/// Counts `view()` calls; the examples print it under the demo grid.
#[derive(Debug, Default)]
pub struct RebuildCounter(Cell<u64>);

impl RebuildCounter {
    /// Starts at zero.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Call once at the top of `view()`.
    pub fn bump(&self) {
        self.0.set(self.0.get() + 1);
    }

    /// Rebuilds so far.
    #[must_use]
    pub fn count(&self) -> u64 {
        self.0.get()
    }
}
