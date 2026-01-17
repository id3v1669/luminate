use std::sync::LazyLock;

use iced::{Background, Border, Color, Padding, Shadow, Vector, border::Radius};

pub static BACKGROUND: LazyLock<Option<Background>> =
    LazyLock::new(|| Some(Background::Color(Color::WHITE)));

pub static HEADER_PADDING: LazyLock<Padding> = LazyLock::new(|| Padding::from([15, 17]));
pub static HEADER_RADIUS: LazyLock<Radius> = LazyLock::new(|| Radius {
    top_left: 25.0,
    top_right: 25.0,
    bottom_left: 0.0,
    bottom_right: 0.0,
});
pub static HEADER_BOTTOM_SHADOW: LazyLock<Shadow> = LazyLock::new(|| Shadow {
    color: Color::from_rgba8(0, 0, 0, 0.1),
    offset: Vector::new(0.0, 1.0),
    blur_radius: 0.0,
});
pub static HEADER_BORDER: LazyLock<Border> = LazyLock::new(|| Border {
    color: Color::TRANSPARENT,
    width: 0.0,
    radius: *HEADER_RADIUS,
});

pub static CONTROLS_PADDING: LazyLock<Padding> = LazyLock::new(|| Padding::from(15));
pub static CONTROLS_RADIUS: LazyLock<Radius> = LazyLock::new(|| Radius {
    top_left: 0.0,
    top_right: 0.0,
    bottom_left: 25.0,
    bottom_right: 25.0,
});
