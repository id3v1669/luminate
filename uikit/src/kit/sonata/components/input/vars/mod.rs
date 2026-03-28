use iced::{Color, Font, Padding, border::Radius};

use crate::kit::sonata::{
    palette::{
        COLOR_FOCUS,
        colors::{gray::SCALE_GRAY, red::SCALE_RED},
        utils::{outer_radius_calc, padding_vh, radius_all},
    },
    utils::text::{
        text::TextStyle,
        vars::{COLOR_TEXT_PRIMARY, COLOR_TEXT_SECONDARY},
    },
};

pub const PADDING_INPUT_HORIZONTAL: Padding = padding_vh(7.0, 8.0);

pub const COLOR_INPUT_BACKGROUND: Color = SCALE_GRAY.s25;
pub const COLOR_INPUT_TEXT: Color = COLOR_TEXT_PRIMARY;
pub const COLOR_INPUT_PLACEHOLDER: Color = SCALE_GRAY.s400;

pub const COLOR_INPUT_BACKGROUND_DISABLED: Color = SCALE_GRAY.s50;

pub const COLOR_INPUT_BORDER: Color = SCALE_GRAY.s100;
pub const COLOR_INPUT_BORDER_FOCUSED: Color = COLOR_FOCUS;

pub const COLOR_INPUT_BORDER_ERROR: Color = SCALE_RED.s500;
pub const COLOR_INPUT_BORDER_OUTER_ERROR: Color = SCALE_RED.s200;

pub const COLOR_INPUT_LABEL_TEXT: Color = COLOR_TEXT_SECONDARY;

pub const COLOR_INPUT_HINT_TEXT: Color = COLOR_TEXT_SECONDARY;
pub const COLOR_INPUT_HINT_TEXT_ERROR: Color = SCALE_RED.s500;

pub const RADIUS_INPUT_BORDER: Radius = radius_all(10.0);

pub const WIDTH_OUTER_RADIUS_MULTIBORDER: f32 = 3.5;
pub const WIDTH_OUTER_OFFSET_MULTIBORDER: f32 = 0.0;
pub const SIZE_OUTER_BORDER: f32 = outer_radius_calc(
    RADIUS_INPUT_BORDER.bottom_left,
    WIDTH_OUTER_RADIUS_MULTIBORDER,
);

const TEXTSTYLE_INPUT: TextStyle = TextStyle::TextMdR;
pub const FONT_INPUT_TEXTSTYLE: Font = TEXTSTYLE_INPUT.build_font();
pub const SIZE_INPUT_TEXTSTYLE: f32 = TEXTSTYLE_INPUT.get_size();

pub const TEXTSTYLE_INPUT_LABEL: TextStyle = TextStyle::TextSmM;
pub const FONT_INPUT_LABEL_TEXTSTYLE: Font = TEXTSTYLE_INPUT_LABEL.build_font();
pub const SIZE_INPUT_LABEL_TEXTSTYLE: f32 = TEXTSTYLE_INPUT_LABEL.get_size();

pub const TEXTSTYLE_INPUT_HINT: TextStyle = TextStyle::TextSmR;
pub const FONT_INPUT_HINT_TEXTSTYLE: Font = TEXTSTYLE_INPUT_HINT.build_font();
pub const SIZE_INPUT_HINT_TEXTSTYLE: f32 = TEXTSTYLE_INPUT_HINT.get_size();

pub const SPACING_INPUT_COLUMN: f32 = 6.0;
