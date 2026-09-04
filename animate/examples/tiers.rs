//! A grid of one-idea animations, each with the line of code that drives it.
//!
//! There is exactly one rule behind all of it:
//!
//! > **An animated value must be read inside a widget, never while building
//! > the view.**
//!
//! `view()` runs when application state changes, on a click, not on a frame.
//! A value read there is a snapshot and would hold still until the next
//! rebuild. So the engine hands out `Anim<T>` handles instead of numbers, and
//! the widget resolves the handle in its own `layout` or `draw`, on the frame
//! it is painting. That is why this page animates without publishing a single
//! message per frame, the rebuild counter at the bottom proves it.
//!
//! | Tier | Wrapper | Reads it in | Cost per frame |
//! |---|---|---|---|
//! | Composite | `Cached` (`iced_texture_cache`) | the compositor | a composite of an existing texture |
//! | Paint | `shape` | `draw` | a redraw |
//! | Layout | `sized` | `layout` | a relayout, then a redraw |
//!
//! The compositor tier is shown by the `compositor` example of
//! `iced_texture_cache`.
//!
//! `ANIM_AUTOPLAY=1` flips the poses every 1.5 s for unattended captures.

use std::time::Instant;

use iced::widget::{button, column, grid, text};
use iced::{Alignment, Element, Length, Subscription};
use iced_animate::{Motion, Presence};
use luminate_examples_support::{Autoplay, CellStyle, Chip, MUTED, RebuildCounter, demo};

fn main() -> iced::Result {
    iced::application(App::new, App::update, App::view)
        .subscription(App::subscription)
        .title("iced_animate: tiers")
        .run()
}

/// Application messages.
#[derive(Debug, Clone, Copy)]
enum Message {
    /// Flips every demo between its two poses. One message controls all nine.
    /// none of them need another one until the next flip.
    Toggle,
    /// Adds a chip to the enter/exit lane.
    AddChip,
    /// Marks a chip as leaving; it stays until its exit animation is gone.
    RemoveChip(u64),
    /// Autoplay only: a frame timestamp.
    Frame(Instant),
}

/// The application: one engine, one pose flag, the chips and the counters.
struct App {
    motion: Motion,
    on: bool,
    chips: Vec<Chip>,
    next_chip: u64,
    rebuilds: RebuildCounter,
    autoplay: Autoplay,
}

impl App {
    fn new() -> Self {
        Self {
            motion: Motion::new(),
            on: false,
            chips: (1..=3).map(Chip::new).collect(),
            next_chip: 4,
            rebuilds: RebuildCounter::new(),
            autoplay: Autoplay::from_env(),
        }
    }

    fn subscription(&self) -> Subscription<Message> {
        self.autoplay.subscription().map(Message::Frame)
    }

    fn update(&mut self, message: Message) {
        // Chips whose exit has already finished are reaped on the next message;
        // nothing publishes a message when an animation ends.
        let motion = self.motion.clone();
        self.chips
            .retain(|chip| !chip.leaving || motion.presence(chip.key()) != Presence::Gone);

        match message {
            Message::Frame(now) => {
                if self.autoplay.tick(now) {
                    self.on = !self.on;
                }
            }
            Message::Toggle => self.on = !self.on,
            Message::AddChip => {
                self.chips.push(Chip::new(self.next_chip));
                self.next_chip += 1;
            }
            // Marked, not dropped: the chip has to stay in the view while it
            // animates out.
            Message::RemoveChip(id) => {
                if let Some(chip) = self.chips.iter_mut().find(|c| c.id == id) {
                    chip.leaving = true;
                }
            }
        }
    }

    fn view(&self) -> Element<'_, Message> {
        let m = &self.motion;
        let on = self.on;
        let style = CellStyle::default();

        self.rebuilds.bump();

        let cells = grid([
            demo::fill(m, on, style),
            demo::radius(m, on, style),
            demo::border(m, on, style),
            demo::size(m, on, style),
            demo::padding(m, on, style),
            demo::property_set(m, on, style),
            demo::spring_vs_ease(m, on, style),
            demo::staggered(m, on, style),
            demo::entering_and_leaving(
                m,
                &self.chips,
                Message::AddChip,
                Message::RemoveChip,
                style,
            ),
        ])
        .columns(3)
        .height(Length::Shrink)
        .spacing(16);

        let page = column![
            text("Motion").size(28),
            text(
                "One button flips every demo between two poses. Nothing below \
                 publishes a message per frame. The engine advances the \
                 values and each widget reads them as it lays out or paints."
            )
            .size(13)
            .color(MUTED),
            button(text(if on { "Reset" } else { "Play" })).on_press(Message::Toggle),
            cells,
            text(format!("view rebuilds: {}", self.rebuilds.count()))
                .size(12)
                .color(MUTED),
        ]
        .spacing(14)
        .padding(20)
        .align_x(Alignment::Start);

        // The host ticks the engine on every redraw and asks for the next
        // frame while anything is moving.
        self.motion.host(page).into()
    }
}
