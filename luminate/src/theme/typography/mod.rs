//! Type styles and the bundled font.
//!
//! A [`TextStyle`] is a size, a line height, a weight and a slant; build one
//! with [`TextStyle::text`] or [`TextStyle::display`] and turn it into an
//! iced font with [`TextStyle::font`]. Named roles such as body, label, and
//! heading are the [`TypographyTheme`] tokens.
//!
//! Every style requests the family [`FAMILY`]. With the default
//! `bundled-font` feature the two Inter faces are embedded as
//! `FONT_INTER` and `FONT_INTER_ITALIC`; load them with
//! [`Luminate::fonts`](crate::Luminate::fonts) and set [`FONT`] as the
//! application's default font. Without the feature, install Inter on the
//! system or the renderer falls back to its default sans-serif.
//!
//! Inter is © 2016 The Inter Project Authors and licensed under the SIL Open
//! Font License 1.1 (`assets/OFL.txt`).

use iced::advanced::text;
use iced::font::{Family, Stretch, Style, Weight};
use iced::widget::text::{LineHeight, Text};
use iced::{Font, Pixels};

/// The upright Inter variable font (weights 100-900, optical sizes 14-32;
/// 880 KB). Family name [`FAMILY`]. See `assets/OFL.txt` for the OFL-1.1 licence.
#[cfg(feature = "bundled-font")]
pub const FONT_INTER: &[u8] = include_bytes!("./assets/InterVariable.ttf");

/// The italic Inter variable font (same axes; 910 KB). Family name
/// [`FAMILY`]; matched by [`TextStyle::italic`]. OFL-1.1, see
/// `assets/OFL.txt`.
#[cfg(feature = "bundled-font")]
pub const FONT_INTER_ITALIC: &[u8] = include_bytes!("./assets/InterVariable-Italic.ttf");

/// Family name every [`TextStyle`] requests: the `name` table entry of the
/// bundled files (`tests/fonts.rs` asserts it resolves).
pub const FAMILY: &str = "Inter Variable";

/// [`FAMILY`] at normal weight and slant: the application's default font
/// (`iced::application(..).default_font(FONT)`).
pub const FONT: Font = Font {
    family: Family::Name(FAMILY),
    weight: Weight::Normal,
    stretch: Stretch::Normal,
    style: Style::Normal,
};

/// Body text sizes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TextSize {
    /// 12 px, 18 px line height.
    Xs,
    /// 14 px, 20 px line height.
    Sm,
    /// 16 px, 24 px line height.
    Md,
    /// 18 px, 28 px line height.
    Lg,
    /// 20 px, 30 px line height.
    Xl,
}

/// Display (heading) sizes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum DisplaySize {
    /// 24 px, 32 px line height.
    Xs,
    /// 30 px, 38 px line height.
    Sm,
    /// 36 px, 44 px line height.
    Md,
    /// 48 px, 60 px line height.
    Lg,
    /// 60 px, 72 px line height.
    Xl,
    /// 72 px, 90 px line height.
    Xxl,
}

/// A complete type style: size, line height, weight and slant.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TextStyle {
    /// Font size in logical pixels.
    pub size: f32,
    /// Line height in logical pixels.
    pub line_height: f32,
    /// Font weight.
    pub weight: Weight,
    /// Upright or italic.
    pub slant: Style,
}

impl TextStyle {
    /// A body text style.
    #[must_use]
    pub const fn text(size: TextSize, weight: Weight) -> Self {
        let (size, line_height) = match size {
            TextSize::Xs => (12.0, 18.0),
            TextSize::Sm => (14.0, 20.0),
            TextSize::Md => (16.0, 24.0),
            TextSize::Lg => (18.0, 28.0),
            TextSize::Xl => (20.0, 30.0),
        };

        Self {
            size,
            line_height,
            weight,
            slant: Style::Normal,
        }
    }

    /// A display (heading) style.
    #[must_use]
    pub const fn display(size: DisplaySize, weight: Weight) -> Self {
        let (size, line_height) = match size {
            DisplaySize::Xs => (24.0, 32.0),
            DisplaySize::Sm => (30.0, 38.0),
            DisplaySize::Md => (36.0, 44.0),
            DisplaySize::Lg => (48.0, 60.0),
            DisplaySize::Xl => (60.0, 72.0),
            DisplaySize::Xxl => (72.0, 90.0),
        };

        Self {
            size,
            line_height,
            weight,
            slant: Style::Normal,
        }
    }

    /// The same style with an italic slant (drawn with
    /// `FONT_INTER_ITALIC`).
    #[must_use]
    pub const fn italic(mut self) -> Self {
        self.slant = Style::Italic;
        self
    }

    /// The `iced::Font` for this style (family [`FAMILY`]).
    #[must_use]
    pub const fn font(self) -> Font {
        Font {
            family: Family::Name(FAMILY),
            weight: self.weight,
            stretch: Stretch::Normal,
            style: self.slant,
        }
    }

    /// The line height as iced expects it.
    #[must_use]
    pub const fn line_height(self) -> LineHeight {
        LineHeight::Absolute(Pixels(self.line_height))
    }

    /// Applies this style to a text widget.
    #[must_use]
    pub fn apply<'a, Theme, Renderer>(
        self,
        text: Text<'a, Theme, Renderer>,
    ) -> Text<'a, Theme, Renderer>
    where
        Theme: iced::widget::text::Catalog + 'a,
        Renderer: text::Renderer<Font = Font>,
    {
        text.font(self.font())
            .size(self.size)
            .line_height(self.line_height())
    }
}

/// A text widget in `style`.
#[must_use]
pub fn styled_text<'a, Theme, Renderer>(
    content: impl text::IntoFragment<'a>,
    style: TextStyle,
) -> Text<'a, Theme, Renderer>
where
    Theme: iced::widget::text::Catalog + 'a,
    Renderer: text::Renderer<Font = Font>,
{
    style.apply(iced::widget::text(content))
}

/// The named type roles a theme is built from.
///
/// The widget tokens ([`ButtonTheme::label`](crate::theme::ButtonTheme::label),
/// [`InputTheme::text_style`](crate::theme::InputTheme::text_style), …) are
/// copied from these when a [`Theme`](crate::theme::Theme) is constructed;
/// change a widget token to override one place, or build a theme from
/// [`Theme::light`](crate::theme::Theme::light) to change them all.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TypographyTheme {
    /// Running text and typed input.
    pub body: TextStyle,
    /// Body size with a medium weight: error messages, emphasised text.
    pub emphasis: TextStyle,
    /// Captions above controls and button labels.
    pub label: TextStyle,
    /// Hints below controls.
    pub caption: TextStyle,
    /// Card headers.
    pub heading: TextStyle,
}

impl TypographyTheme {
    /// The Luminate scale.
    pub const DEFAULT: Self = Self {
        body: TextStyle::text(TextSize::Md, Weight::Normal),
        emphasis: TextStyle::text(TextSize::Md, Weight::Medium),
        label: TextStyle::text(TextSize::Sm, Weight::Medium),
        caption: TextStyle::text(TextSize::Sm, Weight::Normal),
        heading: TextStyle::text(TextSize::Lg, Weight::Semibold),
    };
}

impl Default for TypographyTheme {
    fn default() -> Self {
        Self::DEFAULT
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sizes_and_line_heights_follow_the_scale() {
        let label = TypographyTheme::DEFAULT.label;
        assert_eq!((label.size, label.line_height), (14.0, 20.0));
        assert_eq!(label.weight, Weight::Medium);

        let h = TextStyle::display(DisplaySize::Xxl, Weight::Bold);
        assert_eq!((h.size, h.line_height), (72.0, 90.0));
        assert_eq!(h.font().weight, Weight::Bold);
        assert_eq!(h.line_height(), LineHeight::Absolute(Pixels(90.0)));
    }

    #[test]
    fn italic_changes_only_the_slant() {
        let upright = TextStyle::text(TextSize::Md, Weight::Normal);
        let italic = upright.italic();
        assert_eq!(italic.font().style, Style::Italic);
        assert_eq!(upright.font().style, Style::Normal);
        assert_eq!(italic.font().family, FONT.family);
        assert_eq!(
            (italic.size, italic.line_height, italic.weight),
            (upright.size, upright.line_height, upright.weight)
        );
    }

    #[test]
    fn the_default_font_is_the_family_upright() {
        assert_eq!(FONT.family, Family::Name(FAMILY));
        assert_eq!(FONT, TextStyle::text(TextSize::Md, Weight::Normal).font());
    }

    #[cfg(feature = "bundled-font")]
    #[test]
    fn both_faces_are_truetype() {
        // 0x00010000 is the TrueType sfnt version.
        assert_eq!(&FONT_INTER[..4], &[0, 1, 0, 0]);
        assert_eq!(&FONT_INTER_ITALIC[..4], &[0, 1, 0, 0]);
    }
}
