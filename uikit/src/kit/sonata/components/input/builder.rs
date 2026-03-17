use iced::widget::{Column, Id, container, text, text_input};
use iced::{Background, Color, Renderer, Theme};

use crate::kit::sonata::Sonata;
use crate::kit::sonata::components::input::vars::{
    BORDER_INPUT_DEFAULT, COLOR_INPUT_BACKGROUND, COLOR_INPUT_BACKGROUND_DISABLED,
    COLOR_INPUT_BORDER, PADDING_INPUT_HORIZONTAL,
};
use crate::kit::sonata::components::text::text::TextStyle;
use crate::kit::sonata::components::text::vars::{COLOR_TEXT_PLACEHOLDER, COLOR_TEXT_PRIMARY};
use crate::kit::sonata::palette::COLOR_FOCUS;
use crate::kit::sonata::palette::colors::red::SCALE_RED;
use crate::kit::sonata::utils::multi_border::{self, BorderLayer, multiborder};

fn style(status: text_input::Status) -> text_input::Style {
    let mut style = text_input::Style {
        background: Background::Color(COLOR_INPUT_BACKGROUND),
        border: BORDER_INPUT_DEFAULT,
        icon: Color::BLACK,
        placeholder: COLOR_TEXT_PLACEHOLDER,
        value: COLOR_TEXT_PRIMARY,
        selection: COLOR_FOCUS,
    };

    match status {
        text_input::Status::Disabled => {
            style.background = Background::Color(COLOR_INPUT_BACKGROUND_DISABLED);
            style.border.color = COLOR_INPUT_BORDER;
            style.border.width = 1.0;
        }
        text_input::Status::Active => {
            style.border.color = COLOR_INPUT_BORDER;
            style.border.width = 1.0;
        }
        text_input::Status::Focused { is_hovered: _ } => {
            style.border.color = Color::TRANSPARENT;
            style.border.width = 0.0;
        }
        _ => (),
    }

    style
}

impl<'a, Message> Sonata<Message>
where
    Message: Clone + 'static,
{
    pub fn input(
        &self,
        param: crate::definition::input::UiInput<Message>,
    ) -> iced::Element<'static, Message> {
        let input_id = Id::unique();

        let placeholder = param.placeholder.to_owned();
        let value = param.value.to_owned();

        let mut el: text_input::TextInput<'_, Message, Theme, Renderer> =
            text_input(&placeholder, &value);
        let input_text_style = TextStyle::TextMdR;

        el = el.id(input_id);
        el = el.size(input_text_style.get_size());
        el = el.font(input_text_style.build_font());
        el = el.padding(PADDING_INPUT_HORIZONTAL);

        el = el.style(move |_: &Theme, status| style(status));

        let has_on_input = param.on_input.is_some();

        if let Some(on_param) = param.on_input {
            el = el.on_input(on_param);
        }

        let el = multiborder(el)
            .disabled(!has_on_input)
            .style(|_theme, status| {
                if status.is_disabled {
                    return multi_border::Appearance::new();
                }

                let border_color = if status.is_focused {
                    COLOR_FOCUS
                } else {
                    Color::TRANSPARENT
                };

                multi_border::Appearance::new()
                    .layer(
                        BorderLayer::outer(3.5, border_color)
                            .radius(13.0)
                            .offset(7.0),
                    )
                    .background(Background::Color(SCALE_RED.s300))
            });

        let mut wrapper = container(el);
        wrapper = wrapper.style(|_| container::Style::default());

        let mut column = Column::new();

        if let Some(label) = param.label {
            let label = label.to_owned();
            let label_text_style = TextStyle::TextSmM;

            column = column.push(
                text(label)
                    .font(label_text_style.build_font())
                    .size(label_text_style.get_size())
                    .color(COLOR_TEXT_PRIMARY),
            );
        }

        column = column.push(wrapper);

        if let Some(hint) = param.hint {
            let hint = hint.to_owned();
            column = column.push(
                text(hint)
                    .font(TextStyle::TextSmR.build_font())
                    .size(14)
                    .color(Color::from_rgb8(71, 75, 81)),
            );
        }

        column.spacing(6).into()
    }
}
