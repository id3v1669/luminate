use iced::{Color, Font};

use crate::kit::sonata::{
    palette::colors::red::SCALE_RED,
    utils::text::{text::TextStyle, vars::COLOR_TEXT_PRIMARY},
};

pub const COLOR_TOOLTIP_BACKGROUND: Color = SCALE_RED.s200;
pub const COLOR_TOOLTIP_TEXT: Color = COLOR_TEXT_PRIMARY;

pub const SIZE_TOOLTIP_HORIZONTAL_PADDING: f32 = 10.0;
pub const SIZE_TOOLTIP_VERTICAL_PADDING: f32 = 6.0;
pub const SIZE_TOOLTIP_RIGHT_OFFSET: f32 = 7.0;

pub const FONTTOKEN_TOOLTIP_TEXT: TextStyle = TextStyle::TextMdM;
pub const SIZE_TOOLTIP_TEXT: f32 = FONTTOKEN_TOOLTIP_TEXT.get_size();
pub const FONT_TOOLTIP_TEXT: Font = FONTTOKEN_TOOLTIP_TEXT.build_font();

pub const TOOLTIP_ARROW: &str = r##"
    <svg
        width="44"
        height="22"
        viewBox="0 0 44 22"
        preserveAspectRatio="none"
        xmlns="http://www.w3.org/2000/svg"
    >
        <path
            d="M0,0 C8,0 14,6 18,10 L22,14 C23.5,16 24,18 24,18 C24,18 24.5,16 26,14 L30,10 C34,6 40,0 44,0"
            fill="#FDC3BE"
        />
    </svg>
"##;

pub const SIZE_TOOLTIP_ARROW_WIDTH: f32 = 11.0;
pub const SIZE_TOOLTIP_ARROW_HEIGHT: f32 = 5.5;
pub const SIZE_TOOLTIP_ARROW_RIGHT_OFFSET: f32 = 11.0;
pub const SIZE_TOOLTIP_GAP: f32 = 5.5;
