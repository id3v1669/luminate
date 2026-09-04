//! The compositor tier: `Cached` driven by `iced_animate` values.
//!
//! `Cached` records its content into a texture once and then composites that
//! texture every frame under a transform. Offset, scale and opacity are
//! consumed by the compositor itself, so each frame costs one textured quad.
//! It does not run layout, record again, or redraw the content. The counters
//! at the bottom show how often recording occurs.
//!
//! The engine is reached through this crate's re-export
//! (`iced_texture_cache::iced_animate`), so an application needs one
//! dependency line.
//!
//! `ANIM_AUTOPLAY=1` flips the poses every 1.5 s for unattended captures.

use std::time::Instant;

use iced::widget::{button, column, container, row, text};
use iced::{Alignment, Color, Length, Subscription, Vector};
use iced_texture_cache::iced_animate::curves::SMOOTH;
use iced_texture_cache::iced_animate::{Motion, key};
use iced_texture_cache::{Element, TextureCache, cached};
use luminate_examples_support::{Autoplay, CellStyle, MUTED, RebuildCounter, cell, demo};

/// Fixed-width stages so the cells line up in two columns.
const STYLE: CellStyle = CellStyle {
    stage_width: Length::Fixed(220.0),
    code_first: false,
    code_size: 10.0,
    fill: false,
};

fn main() -> iced::Result {
    // `RUST_LOG=info` shows the adapter and the surface-format choice.
    env_logger::init();

    iced::application(App::new, App::update, App::view)
        .subscription(App::subscription)
        .title("iced_texture_cache: compositor tier")
        .run()
}

/// Application messages.
#[derive(Debug, Clone, Copy)]
enum Message {
    /// Flips every demo between its two poses.
    Toggle,
    /// Autoplay only: a frame timestamp.
    Frame(Instant),
    /// One of the cached buttons was pressed.
    Clicked,
}

/// The application: one engine, one pose flag and the named caches.
struct App {
    motion: Motion,
    on: bool,
    /// One texture per demo. A handle *is* the texture's identity, so it
    /// lives in application state, never in `view()`.
    caches: Caches,
    rebuilds: RebuildCounter,
    autoplay: Autoplay,
    clicks: u32,
}

impl App {
    fn new() -> Self {
        Self {
            motion: Motion::new(),
            on: false,
            caches: Caches::new(),
            rebuilds: RebuildCounter::new(),
            autoplay: Autoplay::from_env(),
            clicks: 0,
        }
    }

    fn subscription(&self) -> Subscription<Message> {
        self.autoplay.subscription().map(Message::Frame)
    }

    fn update(&mut self, message: Message) {
        match message {
            Message::Toggle => self.on = !self.on,
            Message::Clicked => self.clicks += 1,
            Message::Frame(now) => {
                if self.autoplay.tick(now) {
                    self.on = !self.on;
                }
            }
        }
    }

    fn view(&self) -> Element<'_, Message> {
        let m = &self.motion;
        let on = self.on;

        self.rebuilds.bump();

        let cells = column![
            row![
                demo::translate(m, on, &self.caches.translate, STYLE),
                demo::scale(m, on, &self.caches.scale, STYLE),
            ]
            .spacing(16),
            row![
                demo::opacity(m, on, &self.caches.opacity, STYLE),
                self.auto_invalidate_cell(),
            ]
            .spacing(16),
            row![self.nested_cell()].spacing(16),
        ]
        .spacing(16);

        let page = column![
            text("Compositor tier").size(28),
            text(
                "One button flips every demo. Each frame composites an existing \
                 texture. No relayout, re-record, or message per frame."
            )
            .size(13)
            .color(MUTED),
            button(text(if on { "Reset" } else { "Play" })).on_press(Message::Toggle),
            cells,
            text(format!(
                "view rebuilds: {} · records (translate / scale / opacity / button / inner / outer): {}",
                self.rebuilds.count(),
                self.caches.records().join(" / ")
            ))
            .size(12)
            .color(MUTED),
        ]
        .spacing(14)
        .padding(20)
        .align_x(Alignment::Start);

        self.motion.host(page).into()
    }

    /// A cached button: re-records only when the content reacts (hover,
    /// press), while its translation stays a compositor-only animation.
    fn auto_invalidate_cell(&self) -> Element<'_, Message> {
        let offset = self.motion.to(
            key!(),
            SMOOTH,
            if self.on {
                Vector::new(64.0, 0.0)
            } else {
                Vector::ZERO
            },
        );

        cell(
            "auto-invalidate",
            "// re-records only when the\n// content reacts (hover, click)\ncached(cache, button)\n    .translate(offset)",
            cached(
                self.caches.button.clone(),
                button(text(format!("clicked {}", self.clicks))).on_press(Message::Clicked),
            )
            .translate(offset)
            .into(),
            STYLE,
        )
    }

    /// An inner `Cached` baked into an outer one: an inner re-record forces
    /// the outer to re-record in the same frame.
    fn nested_cell(&self) -> Element<'_, Message> {
        cell(
            "nested",
            "// inner re-record forces the\n// outer to re-record\ncached(outer,\n    cached(inner, button))",
            cached(
                self.caches.outer.clone(),
                container(cached(
                    self.caches.inner.clone(),
                    button(text(format!("clicked {}", self.clicks))).on_press(Message::Clicked),
                ))
                .padding(6)
                .style(|_| container::Style {
                    background: Some(iced::Background::Color(Color::from_rgb(0.88, 0.90, 0.94))),
                    ..Default::default()
                }),
            )
            .into(),
            STYLE,
        )
    }
}

/// The named caches, one per `Cached` widget in the view.
struct Caches {
    translate: TextureCache,
    scale: TextureCache,
    opacity: TextureCache,
    /// The auto-invalidate demo's button.
    button: TextureCache,
    /// The nested demo's inner `Cached`…
    inner: TextureCache,
    /// …and the outer one wrapping it.
    outer: TextureCache,
}

impl Caches {
    fn new() -> Self {
        Self {
            translate: TextureCache::new(),
            scale: TextureCache::new(),
            opacity: TextureCache::new(),
            button: TextureCache::new(),
            inner: TextureCache::new(),
            outer: TextureCache::new(),
        }
    }

    fn records(&self) -> Vec<String> {
        [
            &self.translate,
            &self.scale,
            &self.opacity,
            &self.button,
            &self.inner,
            &self.outer,
        ]
        .iter()
        .map(|cache| cache.record_count().to_string())
        .collect()
    }
}
