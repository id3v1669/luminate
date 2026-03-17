use iced::Color;

pub mod colors;
pub mod utils;

pub const fn rgb(r: u8, g: u8, b: u8) -> Color {
    Color::from_rgb8(r, g, b)
}

pub const fn rgba(r: u8, g: u8, b: u8, a: f32) -> Color {
    Color::from_rgba8(r, g, b, a)
}

pub struct ColorScale {
    pub s25: Color,
    pub s30: Color,
    pub s40: Color,
    pub s50: Color,
    pub s100: Color,
    pub s200: Color,
    pub s300: Color,
    pub s400: Color,
    pub s500: Color,
    pub s600: Color,
    pub s700: Color,
    pub s800: Color,
    pub s900: Color,
}

pub const COLOR_ACCENT: Color = rgb(0, 108, 255);
pub const COLOR_WHITE: Color = rgb(255, 255, 255);
pub const COLOR_BLACK: Color = rgb(0, 0, 0);

pub const COLOR_FOCUS: Color = rgba(0, 108, 255, 0.25);
