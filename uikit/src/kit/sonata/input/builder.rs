use iced::border::Radius;
use iced::widget::{Column, Id, container, text, text_input};
use iced::{Background, Border, Color, Padding, Renderer, Theme};

use crate::kit::sonata::Sonata;
use crate::kit::sonata::text::text::TextStyle;

fn style(status: text_input::Status) -> text_input::Style {
    let mut style = text_input::Style {
        background: Background::Color(Color::WHITE),
        border: Border {
            color: Color::from_rgba8(207, 209, 210, 1.0),
            width: 1.0,
            radius: Radius::from(10),
        },
        icon: Color::BLACK,
        placeholder: Color::from_rgb8(139, 142, 148),
        value: Color::from_rgb8(25, 31, 38),
        selection: Color::from_rgba8(0, 108, 255, 0.25),
    };

    match status {
        text_input::Status::Active => {
            style.border.color = Color::from_rgb8(207, 209, 210);
            style.border.width = 1.0;
        }
        text_input::Status::Focused { is_hovered: _ } => {
            style.border.color = Color::from_rgba8(0, 108, 255, 0.25);
            style.border.width = 2.0;
        }
        text_input::Status::Disabled => {
            style.border.color = Color::from_rgb8(207, 209, 210);
            style.border.width = 1.0;
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
        el = el.id(input_id.clone());
        el = el.size(16);
        el = el.padding(Padding::from([8, 7]));

        el = el.style(move |_: &Theme, status| style(status));

        if let Some(on_param) = param.on_input {
            el = el.on_input(on_param);
        }

        let mut wrapper = container(el);
        wrapper = wrapper.style(|_| container::Style::default());

        let mut column = Column::new();

        if let Some(label) = param.label {
            let label = label.to_owned();
            column = column.push(
                text(label)
                    .font(TextStyle::TextSmM.build_font())
                    // Text secondary
                    .color(Color::from_rgb8(71, 75, 81)),
            );
        }

        column = column.push(wrapper);

        if let Some(hint) = param.hint {
            let hint = hint.to_owned();
            column = column.push(
                text(hint)
                    .font(TextStyle::TextSmR.build_font())
                    .color(Color::from_rgb8(71, 75, 81)),
            );
        }

        column.spacing(6).into()
    }
}
