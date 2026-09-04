//! Every stock iced widget the theme has a `Catalog` for builds as an
//! `iced_luminate::Element` and lays out under both shipped looks.

use iced_luminate::iced::widget::{
    checkbox, column, combo_box, pick_list, progress_bar, radio, rule, scrollable, slider,
    text_editor, toggler, vertical_slider,
};
use iced_luminate::iced::{self, Size};
use iced_luminate::theme::Theme;
use iced_luminate::{Element, Luminate, Renderer};
use iced_test::Simulator;

// The payloads only satisfy the widgets' constructors; nothing reads them.
#[allow(dead_code)]
#[derive(Debug, Clone)]
enum Message {
    Checked(bool),
    Toggled(bool),
    Slid(f32),
    Picked(&'static str),
    Chosen(String),
    Edited(text_editor::Action),
}

const OPTIONS: [&str; 3] = ["one", "two", "three"];

fn view<'a>(
    combo: &'a combo_box::State<String>,
    content: &'a text_editor::Content<Renderer>,
) -> Element<'a, Message> {
    column![
        checkbox(true).label("check me").on_toggle(Message::Checked),
        toggler(true).label("toggle me").on_toggle(Message::Toggled),
        slider(0.0..=1.0, 0.5, Message::Slid),
        vertical_slider(0.0..=1.0, 0.5, Message::Slid),
        radio("pick me", 1, Some(1), |_| Message::Checked(true)),
        pick_list(OPTIONS, Some("one"), Message::Picked),
        combo_box(combo, "type here", None, Message::Chosen),
        progress_bar(0.0..=1.0, 0.3),
        text_editor(content).on_action(Message::Edited),
        rule::horizontal(1),
        scrollable(column![checkbox(false).label("inside a scrollable")]),
    ]
    .spacing(8)
    .into()
}

#[test]
fn every_stock_widget_lays_out_under_both_looks() {
    let combo = combo_box::State::new(OPTIONS.iter().map(ToString::to_string).collect());
    let content = text_editor::Content::with_text("some text");

    for theme in [Theme::LIGHT, Theme::DARK] {
        let luminate = Luminate::with_theme(theme);
        let mut ui: Simulator<'_, Message, Theme, Renderer> = Simulator::with_size(
            iced::Settings::default(),
            Size::new(600.0, 800.0),
            luminate.host(view(&combo, &content)),
        );
        let _ = ui.simulate([iced::Event::Window(iced::window::Event::RedrawRequested(
            iced::time::Instant::now(),
        ))]);

        // Only `checkbox` exposes its label as a selectable `Text`; the
        // snapshot below proves the rest draw.
        for label in ["check me", "inside a scrollable"] {
            let _ = ui
                .find(label)
                .unwrap_or_else(|e| panic!("{}: {label}: {e:?}", theme.name));
        }
        let _ = ui.snapshot(&theme).expect("draws under the kit's renderer");
    }
}
