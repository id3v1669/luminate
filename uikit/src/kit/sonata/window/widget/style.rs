use iced::{Background, Color, Theme};

#[derive(Default, Clone, Copy, PartialEq)]
pub struct Style {
    pub background: Option<Background>,
    pub text_color: Option<Color>,
}

pub type StyleFn<'a, Theme> = Box<dyn Fn(&Theme) -> Style + 'a>;

pub trait Catalog {
    type Class<'a>;

    fn default<'a>() -> Self::Class<'a>;

    fn style(&self, class: &Self::Class<'_>) -> Style;
}

impl Catalog for Theme {
    type Class<'a> = StyleFn<'a, Self>;

    fn default<'a>() -> Self::Class<'a> {
        Box::new(|_| Style::default())
    }

    fn style(&self, class: &Self::Class<'_>) -> Style {
        class(self)
    }
}
