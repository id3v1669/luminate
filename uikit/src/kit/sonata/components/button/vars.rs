use iced::{Background, Color};

use crate::kit::sonata::{
    components::text::vars::{COLOR_TEXT_PRIMARY, COLOR_TEXT_SECONDARY},
    palette::{
        COLOR_ACCENT, COLOR_BLACK, COLOR_WHITE,
        colors::{gray::SCALE_GRAY, red::SCALE_RED},
        utils::mix,
    },
};

// Primary
pub const COLOR_BUTTON_TEXT_PRIMARY: Color = COLOR_WHITE;
pub const BACKGROUND_BUTTON_PRIMARY: Background = Background::Color(COLOR_ACCENT);

pub const COLOR_BUTTON_TEXT_PRIMARY_HOVER: Color = COLOR_WHITE;
pub const BACKGROUND_BUTTON_PRIMARY_HOVER: Background =
    Background::Color(mix(COLOR_ACCENT, COLOR_BLACK, 0.1));

pub const COLOR_BUTTON_TEXT_PRIMARY_DISABLED: Color = COLOR_WHITE;
pub const BACKGROUND_BUTTON_PRIMARY_DISABLED: Background =
    Background::Color(mix(COLOR_ACCENT, COLOR_WHITE, 0.5));

// Secondary
pub const COLOR_BUTTON_TEXT_SECONDARY: Color = COLOR_TEXT_PRIMARY;
pub const BACKGROUND_BUTTON_SECONDARY: Background = Background::Color(SCALE_GRAY.s50);

pub const COLOR_BUTTON_TEXT_SECONDARY_HOVER: Color = COLOR_TEXT_PRIMARY;
pub const BACKGROUND_BUTTON_SECONDARY_HOVER: Background =
    Background::Color(mix(SCALE_GRAY.s50, COLOR_BLACK, 0.1));

pub const COLOR_BUTTON_TEXT_SECONDARY_DISABLED: Color = mix(COLOR_TEXT_SECONDARY, COLOR_WHITE, 0.5);
pub const BACKGROUND_BUTTON_SECONDARY_DISABLED: Background =
    Background::Color(mix(SCALE_GRAY.s50, COLOR_WHITE, 0.5));

// Tertiary
pub const COLOR_BUTTON_TEXT_TERTIARY: Color = COLOR_TEXT_SECONDARY;
pub const BACKGROUND_BUTTON_TERTIARY: Background = Background::Color(Color::TRANSPARENT);

pub const COLOR_BUTTON_TEXT_TERTIARY_HOVER: Color = COLOR_TEXT_SECONDARY;
pub const BACKGROUND_BUTTON_TERTIARY_HOVER: Background = Background::Color(Color::TRANSPARENT);

pub const COLOR_BUTTON_TEXT_TERTIARY_DISABLED: Color = mix(COLOR_TEXT_SECONDARY, COLOR_WHITE, 0.5);
pub const BACKGROUND_BUTTON_TERTIARY_DISABLED: Background = Background::Color(Color::TRANSPARENT);

// Destructive
pub const COLOR_BUTTON_TEXT_DESTRUCTIVE: Color = COLOR_WHITE;
pub const BACKGROUND_BUTTON_DESTRUCTIVE: Background = Background::Color(SCALE_RED.s500);

pub const COLOR_BUTTON_TEXT_DESTRUCTIVE_HOVER: Color = COLOR_WHITE;
pub const BACKGROUND_BUTTON_DESTRUCTIVE_HOVER: Background = Background::Color(SCALE_RED.s600);

pub const COLOR_BUTTON_TEXT_DESTRUCTIVE_DISABLED: Color = COLOR_WHITE;
pub const BACKGROUND_BUTTON_DESTRUCTIVE_DISABLED: Background =
    Background::Color(mix(SCALE_RED.s500, COLOR_WHITE, 0.5));
