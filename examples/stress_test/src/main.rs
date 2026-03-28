// examples/stress_test/src/main.rs

use std::collections::VecDeque;
use std::time::{Duration, Instant};

use iced::widget::{Column, Row, button, column, container, row, scrollable, text, text_input};
use iced::{Color, Element, Length, Task, theme};
use iced_auravibe::{Kit, kit::sonata::Sonata, mapper::UIMapper};

fn main() -> iced::Result {
    iced::application(App::new, App::update, App::view)
        .style(|_, _| theme::Style {
            background_color: Color::WHITE,
            text_color: Color::BLACK,
        })
        .run()
}

// ─── Configuration ───

const COUNTS: &[usize] = &[10, 50, 100, 250, 500, 1000, 10_000, 100_000];
const FRAME_HISTORY: usize = 120;

// ════════════════════════════════════════════════════════════
//  COMPONENT REGISTRY
//  Add new testable components here
// ════════════════════════════════════════════════════════════

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ComponentId {
    KitInput,
    KitInputFull,
    KitButton,
    RawTextInput,
    RawButton,
}

impl ComponentId {
    const ALL: &[ComponentId] = &[
        ComponentId::KitInput,
        ComponentId::KitInputFull,
        ComponentId::KitButton,
        ComponentId::RawTextInput,
        ComponentId::RawButton,
    ];

    fn label(self) -> &'static str {
        match self {
            ComponentId::KitInput => "Kit Input",
            ComponentId::KitInputFull => "Kit Input (full)",
            ComponentId::KitButton => "Kit Button",
            ComponentId::RawTextInput => "Raw text_input",
            ComponentId::RawButton => "Raw button",
        }
    }

    /// Build a single instance of the component.
    /// `idx` — element index, `state` — shared app state.
    fn build<'a>(
        self,
        idx: usize,
        state: &'a AppState,
        kit: &UIMapper<'a, Message>,
    ) -> Element<'a, Message> {
        let item = &state.items[idx];

        match self {
            // ── Kit components ──
            ComponentId::KitInput => kit
                .input("Type here...", &item.value)
                .on_input(move |s| Message::InputChanged(idx, s))
                .into(),

            ComponentId::KitInputFull => kit
                .input("Type here...", &item.value)
                .on_input(move |s| Message::InputChanged(idx, s))
                .label(&item.label)
                .hint(&item.hint)
                .is_error(item.is_error)
                .into(),

            ComponentId::KitButton => kit
                .button()
                .label(&item.label)
                .on_press(Message::Noop)
                .into(),

            // ── Baseline (raw iced widgets) ──
            ComponentId::RawTextInput => text_input("Type here...", &item.value)
                .on_input(move |s| Message::InputChanged(idx, s))
                .into(),

            ComponentId::RawButton => button(text(&item.label).size(14))
                .on_press(Message::Noop)
                .into(),
        }
    }
}

// ─── Metrics ───

#[derive(Debug, Clone, Default)]
struct Metrics {
    view_times: VecDeque<Duration>,
    update_times: VecDeque<Duration>,
    frame_times: VecDeque<Duration>,
    last_view: Option<Instant>,
}

impl Metrics {
    fn push_update(&mut self, d: Duration) {
        Self::push(&mut self.update_times, d);
    }

    fn push(buf: &mut VecDeque<Duration>, d: Duration) {
        if buf.len() >= FRAME_HISTORY {
            buf.pop_front();
        }
        buf.push_back(d);
    }

    fn reset(&mut self) {
        self.view_times.clear();
        self.update_times.clear();
        self.frame_times.clear();
        self.last_view = None;
    }

    fn avg_ms(buf: &VecDeque<Duration>) -> f64 {
        if buf.is_empty() {
            return 0.0;
        }
        let sum: Duration = buf.iter().sum();
        sum.as_secs_f64() * 1000.0 / buf.len() as f64
    }

    fn max_ms(buf: &VecDeque<Duration>) -> f64 {
        buf.iter()
            .max()
            .map(|d| d.as_secs_f64() * 1000.0)
            .unwrap_or(0.0)
    }

    fn p99_ms(buf: &VecDeque<Duration>) -> f64 {
        if buf.is_empty() {
            return 0.0;
        }
        let mut sorted: Vec<f64> = buf.iter().map(|d| d.as_secs_f64() * 1000.0).collect();
        sorted.sort_by(|a, b| a.partial_cmp(b).unwrap());
        let idx = ((sorted.len() as f64 * 0.99) as usize).min(sorted.len() - 1);
        sorted[idx]
    }

    fn fps(&self) -> f64 {
        if self.frame_times.is_empty() {
            return 0.0;
        }
        let sum: Duration = self.frame_times.iter().sum();
        let avg = sum.as_secs_f64() / self.frame_times.len() as f64;
        if avg > 0.0 { 1.0 / avg } else { 0.0 }
    }
}

// ─── Item state ───

struct ItemState {
    value: String,
    label: String,
    hint: String,
    is_error: bool,
}

// ─── App state ───

struct AppState {
    items: Vec<ItemState>,
}

impl AppState {
    fn with_count(count: usize) -> Self {
        Self {
            items: (0..count)
                .map(|i| ItemState {
                    value: String::new(),
                    label: format!("Item #{}", i + 1),
                    hint: format!("Hint for item {}", i + 1),
                    is_error: i % 5 == 0,
                })
                .collect(),
        }
    }
}

struct App {
    uikit: Box<dyn for<'a> Kit<'a, Message>>,
    state: AppState,
    metrics: Metrics,
    count: usize,
    active_component: ComponentId,
}

#[derive(Debug, Clone)]
enum Message {
    InputChanged(usize, String),
    SetCount(usize),
    SetComponent(ComponentId),
    ResetMetrics,
    Noop,
}

impl App {
    fn new() -> (Self, Task<Message>) {
        let count = COUNTS[0];
        (
            Self {
                uikit: Box::new(Sonata::new()),
                state: AppState::with_count(count),
                metrics: Metrics::default(),
                count,
                active_component: ComponentId::ALL[0],
            },
            Task::none(),
        )
    }

    fn update(&mut self, message: Message) -> Task<Message> {
        let t = Instant::now();

        match message {
            Message::InputChanged(idx, value) => {
                if let Some(item) = self.state.items.get_mut(idx) {
                    item.value = value;
                }
            }
            Message::SetCount(n) => {
                self.count = n;
                self.state = AppState::with_count(n);
                self.metrics.reset();
            }
            Message::SetComponent(id) => {
                self.active_component = id;
                self.metrics.reset();
            }
            Message::ResetMetrics => self.metrics.reset(),
            Message::Noop => {}
        }

        self.metrics.push_update(t.elapsed());
        Task::none()
    }

    fn view(&self) -> Element<'_, Message> {
        let t = Instant::now();

        let kit = UIMapper::new(&self.uikit);

        // ── Build component list ──
        let mut items = Column::new().spacing(4).width(Length::Fill);
        for i in 0..self.count.min(self.state.items.len()) {
            items = items.push(self.active_component.build(i, &self.state, &kit));
        }

        let left = container(scrollable(items).height(Length::Fill).width(Length::Fill))
            .width(Length::Fill)
            .padding(10);

        // ── Right panel ──
        let right = container(
            column![
                self.view_component_selector(),
                self.view_count_selector(),
                self.view_metrics(t),
            ]
            .spacing(16)
            .width(280),
        )
        .width(Length::Shrink)
        .padding(10);

        container(row![left, right].spacing(10))
            .width(Length::Fill)
            .height(Length::Fill)
            .into()
    }

    // ── Component selector ──

    fn view_component_selector(&self) -> Element<'_, Message> {
        let mut col = Column::new().spacing(4);
        col = col.push(text("── Component ──").size(14));

        let mut row = Row::new().spacing(4);
        for &id in ComponentId::ALL {
            let mut btn = button(text(id.label()).size(11));
            if id != self.active_component {
                btn = btn.on_press(Message::SetComponent(id));
            }
            row = row.push(btn);
        }

        col = col.push(row.wrap());
        col.into()
    }

    // ── Count selector ──

    fn view_count_selector(&self) -> Element<'_, Message> {
        let mut col = Column::new().spacing(4);
        col = col.push(text("── Count ──").size(14));

        let mut row = Row::new().spacing(4);
        for &n in COUNTS {
            let mut btn = button(text(format!("{n}")).size(11));
            if n != self.count {
                btn = btn.on_press(Message::SetCount(n));
            }
            row = row.push(btn);
        }

        col = col.push(row.wrap());
        col.into()
    }

    // ── Metrics panel ──

    fn view_metrics(&self, view_start: Instant) -> Element<'_, Message> {
        let current_ms = view_start.elapsed().as_secs_f64() * 1000.0;
        let m = &self.metrics;

        let r = |ms: f64| {
            if ms < 1.0 {
                "🟢"
            } else if ms < 5.0 {
                "🟡"
            } else if ms < 16.6 {
                "🟠"
            } else {
                "🔴"
            }
        };

        let fr = |fps: f64| {
            if fps >= 55.0 {
                "🟢"
            } else if fps >= 30.0 {
                "🟡"
            } else if fps >= 15.0 {
                "🟠"
            } else {
                "🔴"
            }
        };

        let view_avg = Metrics::avg_ms(&m.view_times);
        let view_p99 = Metrics::p99_ms(&m.view_times);
        let view_max = Metrics::max_ms(&m.view_times);
        let update_avg = Metrics::avg_ms(&m.update_times);
        let fps = m.fps();

        column![
            text("── Metrics ──").size(14),
            text(format!("Component: {}", self.active_component.label())).size(12),
            text(format!("Count: {}", self.count)).size(12),
            text("").size(4),
            text(format!(
                "{} view() current: {current_ms:.3} ms",
                r(current_ms)
            ))
            .size(12),
            text(format!("{} view() avg:     {view_avg:.3} ms", r(view_avg))).size(12),
            text(format!("{} view() p99:     {view_p99:.3} ms", r(view_p99))).size(12),
            text(format!("{} view() max:     {view_max:.3} ms", r(view_max))).size(12),
            text("").size(4),
            text(format!(
                "{} update() avg:   {update_avg:.3} ms",
                r(update_avg)
            ))
            .size(12),
            text("").size(4),
            text(format!("{} FPS:            {fps:.1}", fr(fps))).size(12),
            text("").size(8),
            text("🟢 <1ms  🟡 <5ms  🟠 <16ms  🔴 ≥16ms").size(10),
            text("").size(4),
            button(text("Reset metrics").size(12)).on_press(Message::ResetMetrics),
        ]
        .spacing(2)
        .into()
    }
}
