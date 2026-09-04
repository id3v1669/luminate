//! `Sized` shrinks the box around its content without moving the content,
//! never resolves a negative padding and clips what sticks out below.

use iced::advanced::renderer::Headless;
use iced::advanced::{clipboard, renderer};
use iced::time::Instant;
use iced::widget::container;
use iced::{Color, Event, Font, Length, Padding, Pixels, Rectangle, Settings, Size, mouse, window};
use iced_animate::widget::{shape, sized};
use iced_test::Simulator;
use iced_test::runtime::user_interface::{self, UserInterface};
use iced_test::selector;

const SIZE: Size = Size::new(200.0, 200.0);

fn simulator(root: iced::Element<'_, ()>) -> Simulator<'_, ()> {
    Simulator::with_size(Settings::default(), SIZE, root)
}

fn bounds(ui: &mut Simulator<'_, ()>, id: &'static str) -> Rectangle {
    ui.find(selector::id(id)).expect("laid out").bounds()
}

#[test]
fn a_half_collapsed_box_halves_its_height_and_keeps_its_child_in_place() {
    let root: iced::Element<'_, ()> =
        container(sized(container(shape().width(40.0).height(100.0)).id("inner")).collapse(0.5))
            .id("outer")
            .into();
    let mut ui = simulator(root);

    let outer = bounds(&mut ui, "outer");
    let inner = bounds(&mut ui, "inner");
    assert!((outer.height - 50.0).abs() < 0.5, "outer: {outer:?}");
    assert!(
        (inner.height - 100.0).abs() < 0.5,
        "the child keeps its size: {inner:?}"
    );
    assert!(
        (inner.y - outer.y).abs() < 0.5,
        "the child keeps its position: {inner:?} in {outer:?}"
    );
}

#[test]
fn a_negative_padding_resolves_to_zero() {
    let root: iced::Element<'_, ()> = container(
        sized(container(shape().width(40.0).height(30.0)).id("inner")).padding(Padding::new(-10.0)),
    )
    .id("outer")
    .into();
    let mut ui = simulator(root);

    let outer = bounds(&mut ui, "outer");
    let inner = bounds(&mut ui, "inner");
    assert!(
        (outer.width - inner.width).abs() < 0.5 && (outer.height - inner.height).abs() < 0.5,
        "{outer:?} vs {inner:?}"
    );
    assert!((inner.x - outer.x).abs() < 0.5 && (inner.y - outer.y).abs() < 0.5);
}

#[test]
fn a_fully_open_box_adds_no_clip_and_a_collapsed_one_clips_below_its_edge() {
    let mut renderer = iced_test::futures::futures::executor::block_on(
        <iced::Renderer as Headless>::new(Font::DEFAULT, Pixels(16.0), Some("tiny-skia")),
    )
    .expect("tiny_skia needs no GPU");

    // Red 40 x 100 at the origin, box collapsed to a quarter.
    let root: iced::Element<'_, ()> = container(
        sized(
            shape()
                .width(40.0)
                .height(100.0)
                .fill(Color::from_rgb(1.0, 0.0, 0.0)),
        )
        .collapse(0.25),
    )
    .width(Length::Fill)
    .height(Length::Fill)
    .into();
    let mut ui: UserInterface<'_, (), iced::Theme, iced::Renderer> =
        UserInterface::build(root, SIZE, user_interface::Cache::default(), &mut renderer);
    let mut messages = Vec::new();
    let _ = ui.update(
        &[Event::Window(
            window::Event::RedrawRequested(Instant::now()),
        )],
        mouse::Cursor::Unavailable,
        &mut renderer,
        &mut clipboard::Null,
        &mut messages,
    );
    ui.draw(
        &mut renderer,
        &iced::Theme::Light,
        &renderer::Style {
            text_color: Color::BLACK,
        },
        mouse::Cursor::Unavailable,
    );
    let rgba = renderer.screenshot(Size::new(200, 200), 1.0, Color::WHITE);
    let pixel = |x: usize, y: usize| -> [u8; 3] {
        let i = (y * 200 + x) * 4;
        [rgba[i], rgba[i + 1], rgba[i + 2]]
    };
    assert_eq!(
        pixel(20, 10),
        [255, 0, 0],
        "inside the 25 px the box still shows"
    );
    assert_eq!(
        pixel(20, 60),
        [255, 255, 255],
        "below the collapsed edge the content is clipped"
    );
}
