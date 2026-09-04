//! How a cached subtree decides it must be re-recorded.
//!
//! Shared by [`Cached`](crate::Cached) and [`Pager`](crate::Pager): both run
//! their content against a private [`Shell`] and read back what it recorded.

use iced_core::time::Instant;
use iced_core::{Shell, event, window};

/// What the content's local shell reported for one event.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
struct Reaction {
    /// Requested `NextFrame`, captured the event, invalidated layout or
    /// widgets, or published a message.
    reacted: bool,
    /// Requested `RedrawRequest::At(t)` with `t` already due.
    redraw_due: bool,
    /// A stored `At` deadline was reached on this `RedrawRequested`.
    deadline_reached: bool,
    /// The child's `mouse::Interaction` differs from the previous frame.
    interaction_changed: bool,
}

/// Per-subtree bookkeeping the observer updates.
#[derive(Debug, Clone, Copy, Default)]
pub(crate) struct Activity {
    /// Whether the content reacted / was still animating on the previous
    /// frame. Drives a single trailing re-record on the falling edge.
    pub was_active: bool,
    /// Earliest `RedrawRequest::At` the content asked for and has not yet
    /// been honoured.
    pub deadline: Option<Instant>,
}

/// Pure decision: `(invalidate now, new was_active)`.
///
/// The trailing frame (`was_active` without activity) gives one final
/// re-record so the texture holds the content's resting pose.
fn invalidation_decision(reaction: Reaction, was_active: bool) -> (bool, bool) {
    let active = reaction.reacted
        || reaction.redraw_due
        || reaction.deadline_reached
        || reaction.interaction_changed;

    (active || was_active, active)
}

/// Reads what `local` recorded for one event, folds it into `activity`, and
/// returns whether the subtree must be re-recorded.
pub(crate) fn observe<Message>(
    local: &Shell<'_, Message>,
    redraw_now: Option<Instant>,
    interaction_changed: bool,
    activity: &mut Activity,
) -> bool {
    let redraw = local.redraw_request();

    let reacted = matches!(redraw, window::RedrawRequest::NextFrame)
        || local.event_status() == event::Status::Captured
        || local.is_layout_invalid()
        || local.are_widgets_invalid()
        || !local.is_empty();

    let deadline_reached = matches!(
        (redraw_now, activity.deadline),
        (Some(now), Some(deadline)) if now >= deadline
    );

    if deadline_reached {
        activity.deadline = None;
    }

    let mut redraw_due = false;
    if let window::RedrawRequest::At(t) = redraw {
        match redraw_now {
            Some(now) if t <= now => redraw_due = true,
            _ => activity.deadline = Some(activity.deadline.map_or(t, |d| d.min(t))),
        }
    }

    let (invalidate, was_active) = invalidation_decision(
        Reaction {
            reacted,
            redraw_due,
            deadline_reached,
            interaction_changed,
        },
        activity.was_active,
    );

    activity.was_active = was_active;
    invalidate
}

#[cfg(test)]
mod tests {
    use super::*;

    fn quiet() -> Reaction {
        Reaction::default()
    }

    #[test]
    fn a_quiet_frame_does_not_invalidate() {
        assert_eq!(invalidation_decision(quiet(), false), (false, false));
    }

    #[test]
    fn a_reaction_invalidates_and_arms_the_trailing_frame() {
        let reaction = Reaction {
            reacted: true,
            ..quiet()
        };
        assert_eq!(invalidation_decision(reaction, false), (true, true));
        // The next quiet frame re-records once more, then stops.
        assert_eq!(invalidation_decision(quiet(), true), (true, false));
        assert_eq!(invalidation_decision(quiet(), false), (false, false));
    }

    #[test]
    fn deadline_and_interaction_changes_count_as_activity() {
        for reaction in [
            Reaction {
                deadline_reached: true,
                ..quiet()
            },
            Reaction {
                redraw_due: true,
                ..quiet()
            },
            Reaction {
                interaction_changed: true,
                ..quiet()
            },
        ] {
            assert!(invalidation_decision(reaction, false).0);
        }
    }

    #[test]
    fn a_future_deadline_is_stored_and_fires_when_due() {
        let mut messages: Vec<()> = Vec::new();
        let mut activity = Activity::default();
        let now = Instant::now();
        let later = now + std::time::Duration::from_millis(50);

        let mut shell = Shell::new(&mut messages);
        shell.request_redraw_at(later);
        assert!(!observe(&shell, Some(now), false, &mut activity));
        assert_eq!(activity.deadline, Some(later));

        let quiet_shell = Shell::new(&mut messages);
        assert!(
            observe(&quiet_shell, Some(later), false, &mut activity),
            "the stored deadline fires on the frame that reaches it"
        );
        assert_eq!(activity.deadline, None);
    }
}
