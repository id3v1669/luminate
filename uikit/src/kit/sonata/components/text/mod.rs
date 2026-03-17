use iced::{
    Font,
    font::{Family, Stretch, Style, Weight},
};

pub mod display;
pub mod text;
pub mod vars;

pub const FONT_INTER: &[u8] = include_bytes!("./assets/Inter-Regular.ttf");

pub struct FontToken {
    pub weight: Weight,
    pub style: Style,
    pub size: f32,
}

impl<'a> display::DisplayStyle {
    const fn token(self) -> &'a FontToken {
        &display::TOKENS[self as usize]
    }

    pub fn build_font(self) -> Font {
        let token = Self::token(self);

        Font {
            family: Family::Name("Inter"),
            weight: token.weight,
            stretch: Stretch::Normal,
            style: token.style,
        }
    }

    pub fn get_size(self) -> f32 {
        let token = Self::token(self);
        token.size
    }
}

impl<'a> text::TextStyle {
    const fn token(self) -> &'a FontToken {
        &text::TOKENS[self as usize]
    }

    pub fn build_font(self) -> Font {
        let token = Self::token(self);

        Font {
            family: Family::Name("Inter"),
            weight: token.weight,
            stretch: Stretch::Normal,
            style: token.style,
        }
    }

    pub fn get_size(self) -> f32 {
        let token = Self::token(self);
        token.size
    }
}
