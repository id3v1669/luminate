use iced::Alignment::Center;
use iced::Length::Fill;
use iced::border::Radius;
use iced::widget::button;
use iced::widget::button::Status;
use iced::widget::text;
use iced::{Border, Color, Element, Padding, Shadow, Vector, widget};

use crate::UiButton;
use crate::definition::button::props::{ButtonHierarchy, ButtonSize};
use crate::kit::sonata::Sonata;
use crate::kit::sonata::components::button::vars::{
    BACKGROUND_BUTTON_DESTRUCTIVE, BACKGROUND_BUTTON_DESTRUCTIVE_DISABLED,
    BACKGROUND_BUTTON_DESTRUCTIVE_HOVER, BACKGROUND_BUTTON_PRIMARY,
    BACKGROUND_BUTTON_PRIMARY_DISABLED, BACKGROUND_BUTTON_PRIMARY_HOVER,
    BACKGROUND_BUTTON_SECONDARY, BACKGROUND_BUTTON_SECONDARY_DISABLED,
    BACKGROUND_BUTTON_SECONDARY_HOVER, BACKGROUND_BUTTON_TERTIARY,
    BACKGROUND_BUTTON_TERTIARY_DISABLED, BACKGROUND_BUTTON_TERTIARY_HOVER,
    COLOR_BUTTON_TEXT_DESTRUCTIVE, COLOR_BUTTON_TEXT_DESTRUCTIVE_DISABLED,
    COLOR_BUTTON_TEXT_DESTRUCTIVE_HOVER, COLOR_BUTTON_TEXT_PRIMARY,
    COLOR_BUTTON_TEXT_PRIMARY_DISABLED, COLOR_BUTTON_TEXT_PRIMARY_HOVER,
    COLOR_BUTTON_TEXT_SECONDARY, COLOR_BUTTON_TEXT_SECONDARY_DISABLED,
    COLOR_BUTTON_TEXT_SECONDARY_HOVER, COLOR_BUTTON_TEXT_TERTIARY,
    COLOR_BUTTON_TEXT_TERTIARY_DISABLED, COLOR_BUTTON_TEXT_TERTIARY_HOVER,
};
use crate::kit::sonata::components::text::text::TextStyle;
use crate::kit::sonata::palette::COLOR_FOCUS;
use crate::kit::sonata::palette::colors::red::SCALE_RED;
use crate::kit::sonata::utils::multi_border::{Appearance, BorderLayer, multiborder};

mod vars;

fn define_padding(size: &ButtonSize) -> Padding {
    match size {
        ButtonSize::SM => Padding {
            top: 7.0,
            right: 15.0,
            bottom: 7.0,
            left: 15.0,
        },
        ButtonSize::MD => Padding {
            top: 9.0,
            right: 17.0,
            bottom: 9.0,
            left: 17.0,
        },
        ButtonSize::LG => Padding {
            top: 11.0,
            right: 19.0,
            bottom: 11.0,
            left: 19.0,
        },
    }
}

fn style(status: Status, hier: Option<&ButtonHierarchy>) -> widget::button::Style {
    let hier = hier.unwrap_or(&ButtonHierarchy::Primary);
    let mut style = widget::button::Style::default();

    match hier {
        ButtonHierarchy::Primary => match status {
            Status::Active => {
                style.background = Some(BACKGROUND_BUTTON_PRIMARY);
                style.text_color = COLOR_BUTTON_TEXT_PRIMARY;
            }
            Status::Hovered => {
                style.background = Some(BACKGROUND_BUTTON_PRIMARY_HOVER);
                style.text_color = COLOR_BUTTON_TEXT_PRIMARY_HOVER;
            }
            Status::Pressed => {
                style.background = Some(BACKGROUND_BUTTON_PRIMARY);
                style.text_color = COLOR_BUTTON_TEXT_PRIMARY;
            }
            Status::Disabled => {
                style.background = Some(BACKGROUND_BUTTON_PRIMARY_DISABLED);
                style.text_color = COLOR_BUTTON_TEXT_PRIMARY_DISABLED;
            }
        },
        ButtonHierarchy::Secondary => match status {
            Status::Active => {
                style.background = Some(BACKGROUND_BUTTON_SECONDARY);
                style.text_color = COLOR_BUTTON_TEXT_SECONDARY;
            }
            Status::Hovered => {
                style.background = Some(BACKGROUND_BUTTON_SECONDARY_HOVER);
                style.text_color = COLOR_BUTTON_TEXT_SECONDARY_HOVER;
            }
            Status::Pressed => {
                style.background = Some(BACKGROUND_BUTTON_SECONDARY);
                style.text_color = COLOR_BUTTON_TEXT_SECONDARY;
            }
            Status::Disabled => {
                style.background = Some(BACKGROUND_BUTTON_SECONDARY_DISABLED);
                style.text_color = COLOR_BUTTON_TEXT_SECONDARY_DISABLED;
            }
        },
        ButtonHierarchy::Tertiary => match status {
            Status::Active => {
                style.background = Some(BACKGROUND_BUTTON_TERTIARY);
                style.text_color = COLOR_BUTTON_TEXT_TERTIARY;
            }
            Status::Hovered => {
                style.background = Some(BACKGROUND_BUTTON_TERTIARY_HOVER);
                style.text_color = COLOR_BUTTON_TEXT_TERTIARY_HOVER;
            }
            Status::Pressed => {
                style.background = Some(BACKGROUND_BUTTON_TERTIARY);
                style.text_color = COLOR_BUTTON_TEXT_TERTIARY;
            }
            Status::Disabled => {
                style.background = Some(BACKGROUND_BUTTON_TERTIARY_DISABLED);
                style.text_color = COLOR_BUTTON_TEXT_TERTIARY_DISABLED;
            }
        },
        ButtonHierarchy::Destructive => match status {
            Status::Active => {
                style.background = Some(BACKGROUND_BUTTON_DESTRUCTIVE);
                style.text_color = COLOR_BUTTON_TEXT_DESTRUCTIVE;
            }
            Status::Hovered => {
                style.background = Some(BACKGROUND_BUTTON_DESTRUCTIVE_HOVER);
                style.text_color = COLOR_BUTTON_TEXT_DESTRUCTIVE_HOVER;
            }
            Status::Pressed => {
                style.background = Some(BACKGROUND_BUTTON_DESTRUCTIVE);
                style.text_color = COLOR_BUTTON_TEXT_DESTRUCTIVE;
            }
            Status::Disabled => {
                style.background = Some(BACKGROUND_BUTTON_DESTRUCTIVE_DISABLED);
                style.text_color = COLOR_BUTTON_TEXT_DESTRUCTIVE_DISABLED;
            }
        },
    }

    style.border = Border {
        color: COLOR_FOCUS,
        radius: Radius::from(10),
        ..Default::default()
    };

    if hier == &ButtonHierarchy::Primary && status != Status::Disabled {
        style.shadow = Shadow {
            color: Color::from_rgba8(50, 145, 233, 0.5),
            offset: Vector::ZERO,
            blur_radius: 10.0,
        };
    }

    style
}

impl<Message: Clone + 'static> Sonata<Message> {
    pub fn button(&self, params: UiButton<Message>) -> Element<'static, Message> {
        let UiButton {
            label,
            on_press,
            props,
            width,
            ..
        } = params;

        let size = props.get_size().clone();
        let hier = props.get_hier().clone();

        let mut button = button(
            text(label)
                .font(TextStyle::build_font(TextStyle::TextSmM))
                .align_x(Center)
                .width(Fill)
                .size(14),
        )
        .padding(define_padding(&size))
        .width(width)
        .style(move |_, status| style(status, Some(&hier)));

        if let Some(event) = on_press {
            button = button.on_press(event);
        }

        multiborder(button)
            .style(move |_, status| {
                if status.is_pressed && !status.is_disabled {
                    if props.get_hier() != &ButtonHierarchy::Destructive {
                        Appearance::new().layer(
                            BorderLayer::outer(2.0, COLOR_FOCUS)
                                .offset(2.0)
                                .radius(13.0),
                        )
                    } else {
                        Appearance::new().layer(
                            BorderLayer::outer(2.0, SCALE_RED.s100)
                                .offset(2.0)
                                .radius(13.0),
                        )
                    }
                } else {
                    Appearance::new()
                }
            })
            .into()
    }
}
