use iced::{Border, Color, Padding, border::Radius};

use crate::kit::sonata::palette::{
    colors::gray::SCALE_GRAY,
    utils::{padding_vh, radius_all},
};

pub const PADDING_INPUT_HORIZONTAL: Padding = padding_vh(7.0, 8.0);

pub const COLOR_INPUT_BACKGROUND: Color = SCALE_GRAY.s25;
pub const COLOR_INPUT_BACKGROUND_DISABLED: Color = SCALE_GRAY.s50;
pub const COLOR_INPUT_BORDER: Color = SCALE_GRAY.s100;

pub const RADIUS_INPUT_BORDER: Radius = radius_all(10.0);

pub const BORDER_INPUT_DEFAULT: Border = Border {
    color: COLOR_INPUT_BORDER,
    width: 1.0,
    radius: RADIUS_INPUT_BORDER,
};
