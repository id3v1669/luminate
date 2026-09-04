//! Linear back/forward history of page indices.

use std::collections::VecDeque;

/// A bounded history that never records the same page twice in a row.
///
/// Semantics (also documented on [`Router`](crate::Router)):
///
/// - `push` truncates the forward branch first, then appends the page unless
///   it is already the last entry. Pushing the page under the cursor after a
///   `back()` therefore *does* discard the forward branch, like a browser.
/// - `replace` swaps the entry under the cursor (forward branch discarded).
/// - When the bound is exceeded the oldest entry is forgotten.
/// - `[A, B, A]` is a valid history; only consecutive duplicates collapse.
#[derive(Debug)]
pub(crate) struct History {
    entries: VecDeque<usize>,
    cursor: usize,
    max_len: usize,
}

impl History {
    /// Creates an empty history that keeps at most `max_len` entries.
    ///
    /// # Panics
    ///
    /// Debug builds only: panics if `max_len == 0`. Release builds log an
    /// error and keep one entry.
    #[must_use]
    pub(crate) fn new(max_len: usize) -> Self {
        debug_assert!(max_len > 0, "history must hold at least one entry");

        if max_len == 0 {
            log::error!("history length 0 requested; keeping one entry");
        }

        Self {
            entries: VecDeque::new(),
            cursor: 0,
            max_len: max_len.max(1),
        }
    }

    /// Records a visit to `page`.
    pub(crate) fn push(&mut self, page: usize) {
        self.entries.truncate(self.cursor + 1);

        if self.entries.back() == Some(&page) {
            return;
        }

        self.entries.push_back(page);

        if self.entries.len() > self.max_len {
            let _ = self.entries.pop_front();
        }

        self.cursor = self.entries.len() - 1;
    }

    /// Replaces the entry under the cursor with `page`.
    pub(crate) fn replace(&mut self, page: usize) {
        self.entries.truncate(self.cursor + 1);
        let _ = self.entries.pop_back();
        self.cursor = self.entries.len().saturating_sub(1);
        self.push(page);
    }

    /// The id under the cursor.
    #[must_use]
    pub(crate) fn current(&self) -> Option<usize> {
        self.entries.get(self.cursor).copied()
    }

    #[must_use]
    pub(crate) fn can_go_back(&self) -> bool {
        self.cursor > 0
    }

    #[must_use]
    pub(crate) fn can_go_forward(&self) -> bool {
        self.cursor + 1 < self.entries.len()
    }

    /// Moves the cursor one step back and returns the id there.
    pub(crate) fn back(&mut self) -> Option<usize> {
        if self.can_go_back() {
            self.cursor -= 1;
            self.current()
        } else {
            None
        }
    }

    /// Moves the cursor one step forward and returns the id there.
    pub(crate) fn forward(&mut self) -> Option<usize> {
        if self.can_go_forward() {
            self.cursor += 1;
            self.current()
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn push_after_back_truncates_the_forward_branch() {
        let mut h = History::new(10);
        h.push(0);
        h.push(1);
        h.push(2);
        assert_eq!(h.back(), Some(1));
        h.push(3);
        assert!(!h.can_go_forward());
        assert_eq!(h.current(), Some(3));
        assert_eq!(h.back(), Some(1));
        assert_eq!(h.back(), Some(0));
    }

    #[test]
    fn pushing_the_page_under_the_cursor_after_back_drops_the_forward_branch() {
        let mut h = History::new(10);
        h.push(0);
        h.push(1);
        assert_eq!(h.back(), Some(0));
        h.push(0);
        assert_eq!(h.current(), Some(0));
        assert_eq!(h.forward(), None);
    }

    #[test]
    fn consecutive_duplicates_collapse_but_a_b_a_is_kept() {
        let mut h = History::new(10);
        h.push(0);
        h.push(0);
        assert!(!h.can_go_back());
        h.push(1);
        h.push(0);
        assert_eq!(h.back(), Some(1));
        assert_eq!(h.back(), Some(0));
    }

    #[test]
    fn back_at_start_and_forward_at_end_are_none() {
        let mut h = History::new(10);
        assert_eq!(h.back(), None);
        h.push(0);
        assert_eq!(h.forward(), None);
        assert!(!h.can_go_back());
        assert!(!h.can_go_forward());
    }

    #[test]
    fn history_is_bounded() {
        let mut h = History::new(3);
        for i in 0..10 {
            h.push(i);
        }
        assert_eq!(h.current(), Some(9));
        assert_eq!(h.back(), Some(8));
        assert_eq!(h.back(), Some(7));
        assert_eq!(h.back(), None);
    }

    #[test]
    fn replace_swaps_the_current_entry() {
        let mut h = History::new(10);
        h.replace(5);
        assert_eq!(h.current(), Some(5));
        h.push(1);
        h.replace(2);
        assert_eq!(h.current(), Some(2));
        assert_eq!(h.back(), Some(5));
        assert_eq!(h.forward(), Some(2));
        // Replacing with the previous entry collapses the duplicate.
        h.replace(5);
        assert_eq!(h.current(), Some(5));
        assert!(!h.can_go_back());
    }

    #[test]
    #[cfg(debug_assertions)]
    #[should_panic(expected = "at least one entry")]
    fn a_zero_length_history_panics() {
        let _ = History::new(0);
    }

    #[test]
    #[cfg(not(debug_assertions))]
    fn a_zero_length_history_keeps_one_entry_in_release() {
        let mut h = History::new(0);
        h.push(1);
        h.push(2);
        assert_eq!(h.current(), Some(2));
        assert!(!h.can_go_back());
    }
}
