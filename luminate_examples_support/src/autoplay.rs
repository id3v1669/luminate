//! Unattended captures: flip a demo on a timer instead of on a click.

use std::time::{Duration, Instant};

use iced::Subscription;

/// The environment variable that enables autoplay when set to `1`.
pub const ENV: &str = "ANIM_AUTOPLAY";

const PERIOD: Duration = Duration::from_millis(1500);

/// Fires once every 1.5 s while enabled.
///
/// Read the environment once ([`Autoplay::from_env`]) in `main` or in the
/// application's constructor. Do not read it inside `subscription()`, which runs on
/// every update cycle.
#[derive(Debug, Clone, Copy)]
pub struct Autoplay {
    enabled: bool,
    last: Option<Instant>,
}

impl Autoplay {
    /// Autoplay that is on when `enabled` is `true`.
    #[must_use]
    pub fn new(enabled: bool) -> Self {
        Self {
            enabled,
            last: None,
        }
    }

    /// Reads [`ENV`] once.
    #[must_use]
    pub fn from_env() -> Self {
        Self::new(std::env::var(ENV).is_ok_and(|value| value == "1"))
    }

    /// Whether autoplay is on.
    #[must_use]
    pub fn enabled(&self) -> bool {
        self.enabled
    }

    /// Frame timestamps while enabled; nothing otherwise.
    ///
    /// Map the `Instant` to the example's own message and hand it to
    /// [`Autoplay::tick`].
    pub fn subscription(&self) -> Subscription<Instant> {
        if self.enabled {
            iced::window::frames()
        } else {
            Subscription::none()
        }
    }

    /// Returns `true` the first time and then once per period.
    pub fn tick(&mut self, now: Instant) -> bool {
        let due = self
            .last
            .is_none_or(|last| now.duration_since(last) >= PERIOD);
        if due {
            self.last = Some(now);
        }
        due
    }
}
