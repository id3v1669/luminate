use iced::{Background, Color, Padding, Shadow, Vector};

use crate::kit::sonata::{
    palette::{
        COLOR_ACCENT, COLOR_BLACK, COLOR_FOCUS, COLOR_WHITE,
        colors::{gray::SCALE_GRAY, red::SCALE_RED},
        utils::{mix, padding_vh},
    },
    utils::text::vars::{COLOR_TEXT_PRIMARY, COLOR_TEXT_SECONDARY},
};

pub struct ButtonStatusColors {
    pub background: Background,
    pub text: Color,
}

pub struct ButtonHierarchyStyle {
    pub active: ButtonStatusColors,
    pub hover: ButtonStatusColors,
    pub disabled: ButtonStatusColors,
    pub focus_border_color: Color,
    pub has_shadow: bool,
}

impl ButtonHierarchyStyle {
    pub const fn pressed(&self) -> &ButtonStatusColors {
        &self.hover
    }
}

pub const STYLE_PRIMARY: ButtonHierarchyStyle = ButtonHierarchyStyle {
    active: ButtonStatusColors {
        background: Background::Color(COLOR_ACCENT),
        text: COLOR_WHITE,
    },
    hover: ButtonStatusColors {
        background: Background::Color(mix(COLOR_ACCENT, COLOR_BLACK, 0.1)),
        text: COLOR_WHITE,
    },
    disabled: ButtonStatusColors {
        background: Background::Color(mix(COLOR_ACCENT, COLOR_WHITE, 0.5)),
        text: COLOR_WHITE,
    },
    focus_border_color: COLOR_FOCUS,
    has_shadow: true,
};

pub const STYLE_SECONDARY: ButtonHierarchyStyle = ButtonHierarchyStyle {
    active: ButtonStatusColors {
        background: Background::Color(SCALE_GRAY.s50),
        text: COLOR_TEXT_PRIMARY,
    },
    hover: ButtonStatusColors {
        background: Background::Color(mix(SCALE_GRAY.s50, COLOR_BLACK, 0.1)),
        text: COLOR_TEXT_PRIMARY,
    },
    disabled: ButtonStatusColors {
        background: Background::Color(mix(SCALE_GRAY.s50, COLOR_WHITE, 0.5)),
        text: mix(COLOR_TEXT_SECONDARY, COLOR_WHITE, 0.5),
    },
    focus_border_color: COLOR_FOCUS,
    has_shadow: false,
};

pub const STYLE_TERTIARY: ButtonHierarchyStyle = ButtonHierarchyStyle {
    active: ButtonStatusColors {
        background: Background::Color(Color::TRANSPARENT),
        text: COLOR_TEXT_SECONDARY,
    },
    hover: ButtonStatusColors {
        background: Background::Color(Color::TRANSPARENT),
        text: COLOR_TEXT_SECONDARY,
    },
    disabled: ButtonStatusColors {
        background: Background::Color(Color::TRANSPARENT),
        text: mix(COLOR_TEXT_SECONDARY, COLOR_WHITE, 0.5),
    },
    focus_border_color: COLOR_FOCUS,
    has_shadow: false,
};

pub const STYLE_DESTRUCTIVE: ButtonHierarchyStyle = ButtonHierarchyStyle {
    active: ButtonStatusColors {
        background: Background::Color(SCALE_RED.s500),
        text: COLOR_WHITE,
    },
    hover: ButtonStatusColors {
        background: Background::Color(SCALE_RED.s600),
        text: COLOR_WHITE,
    },
    disabled: ButtonStatusColors {
        background: Background::Color(mix(SCALE_RED.s500, COLOR_WHITE, 0.5)),
        text: COLOR_WHITE,
    },
    focus_border_color: SCALE_RED.s100,
    has_shadow: false,
};

// # Sizes
pub struct ButtonPaddingConfig {
    pub default: Padding,
    pub icon_only: Padding,
    pub combined: Padding,
}

pub const PADDING_BUTTON_SM: ButtonPaddingConfig = ButtonPaddingConfig {
    default: padding_vh(7.0, 15.0),
    icon_only: padding_vh(5.0, 5.0),
    combined: padding_vh(7.0, 10.0),
};

pub const PADDING_BUTTON_MD: ButtonPaddingConfig = ButtonPaddingConfig {
    default: padding_vh(9.0, 17.0),
    icon_only: padding_vh(7.0, 7.0),
    combined: padding_vh(9.0, 7.0),
};

pub const PADDING_BUTTON_LG: ButtonPaddingConfig = ButtonPaddingConfig {
    default: padding_vh(11.0, 19.0),
    icon_only: padding_vh(9.0, 9.0),
    combined: padding_vh(11.0, 11.0),
};

// # General

pub const RADIUS_BUTTON: f32 = 10.0;
pub const FOCUS_BORDER_WIDTH: f32 = 2.0;
pub const FOCUS_BORDER_OFFSET: f32 = 2.0;
pub const FOCUS_BORDER_RADIUS: f32 = 13.0;

pub const SHADOW_PRIMARY: Shadow = Shadow {
    color: Color::from_rgba8(50, 145, 233, 0.5),
    offset: Vector::ZERO,
    blur_radius: 10.0,
};
