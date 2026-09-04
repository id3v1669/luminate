//! `Pager`: pages of different heights that slide into place.
//!
//! While sliding, each visible page is recorded once into its own texture and
//! composited under the slide; the pager's height interpolates between the
//! outgoing and incoming page (layout tier). At rest the page is drawn
//! directly, snapped to the device grid.
//!
//! `ANIM_AUTOPLAY=1` advances the page every 1.5 s for unattended captures.

use std::time::Instant;

use iced::widget::{Space, button, column, container, row, text};
use iced::{Alignment, Background, Border, Color, Length, Subscription};
use iced_texture_cache::iced_animate::Motion;
use iced_texture_cache::{Element, Pager};
use luminate_examples_support::{Autoplay, RebuildCounter};

/// How many pages the pager holds.
const PAGES: usize = 3;

fn main() -> iced::Result {
    // `RUST_LOG=info` shows the adapter and the surface-format choice.
    env_logger::init();

    iced::application(App::new, App::update, App::view)
        .subscription(App::subscription)
        .title("iced_texture_cache: pager")
        .run()
}

/// Application messages.
#[derive(Debug, Clone, Copy)]
enum Message {
    /// Previous page.
    Back,
    /// Next page.
    Next,
    /// The button on page `i` was pressed.
    Clicked(usize),
    /// Autoplay only: a frame timestamp.
    Frame(Instant),
}

/// The application: one engine, the current page and per-page click counts.
struct App {
    motion: Motion,
    page: usize,
    clicks: [u32; PAGES],
    rebuilds: RebuildCounter,
    autoplay: Autoplay,
}

impl App {
    fn new() -> Self {
        Self {
            motion: Motion::new(),
            page: 0,
            clicks: [0; PAGES],
            rebuilds: RebuildCounter::new(),
            autoplay: Autoplay::from_env(),
        }
    }

    fn subscription(&self) -> Subscription<Message> {
        self.autoplay.subscription().map(Message::Frame)
    }

    fn update(&mut self, message: Message) {
        match message {
            Message::Back => self.page = self.page.saturating_sub(1),
            Message::Next => self.page = (self.page + 1).min(PAGES - 1),
            Message::Clicked(i) => self.clicks[i] += 1,
            Message::Frame(now) => {
                if self.autoplay.tick(now) {
                    self.page = (self.page + 1) % PAGES;
                }
            }
        }
    }

    fn view(&self) -> Element<'_, Message> {
        self.rebuilds.bump();

        let pages = (0..PAGES).map(|i| page(i, self.clicks[i]));

        let pager = Pager::new(pages)
            .current(self.page)
            .motion(self.motion.clone())
            .width(420.0);

        let controls = row![
            button("Back").on_press_maybe((self.page > 0).then_some(Message::Back)),
            button("Next").on_press_maybe((self.page + 1 < PAGES).then_some(Message::Next)),
            text(format!("page {} / {}", self.page + 1, PAGES)),
        ]
        .spacing(10)
        .align_y(Alignment::Center);

        let content = column![
            text("Pager").size(28),
            controls,
            container(pager).style(|_| container::Style {
                border: Border {
                    color: Color::from_rgb(0.4, 0.4, 0.45),
                    width: 1.0,
                    radius: 8.0.into(),
                },
                ..Default::default()
            }),
            text(format!("view rebuilds: {}", self.rebuilds.count()))
                .size(12)
                .color(Color::from_rgb(0.6, 0.6, 0.65)),
        ]
        .spacing(14)
        .padding(20);

        self.motion.host(content).into()
    }
}

/// Page `i` is `120 + 80·i` px tall so the pager visibly grows and shrinks.
fn page<'a>(i: usize, clicks: u32) -> Element<'a, Message> {
    let hue = i as f32 / PAGES as f32;

    container(
        column![
            text(format!("Page {}", i + 1)).size(20),
            Space::new().height(Length::Fill),
            button(text(format!("clicked {clicks}"))).on_press(Message::Clicked(i)),
        ]
        .spacing(10)
        .align_x(Alignment::Center),
    )
    .width(Length::Fill)
    .height(Length::Fixed(120.0 + 80.0 * i as f32))
    .padding(16)
    .style(move |_| container::Style {
        background: Some(Background::Color(Color::from_rgb(
            0.25 + 0.5 * hue,
            0.55 - 0.25 * hue,
            0.6,
        ))),
        border: Border {
            radius: 8.0.into(),
            ..Default::default()
        },
        ..Default::default()
    })
    .into()
}
