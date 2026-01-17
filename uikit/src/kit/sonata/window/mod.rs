use iced::{
    Background, Border, Color, Element,
    Length::{Fill, Shrink},
    Padding, Shadow, Theme,
    alignment::{Horizontal, Vertical},
    border::Radius,
    widget::{
        Space, center, column, container, mouse_area, opaque, scrollable,
        scrollable::{AutoScroll, Direction, Rail, Scrollbar, Scroller},
        space, stack, text,
    },
};

use crate::{
    definition::{COMPONENT_DEBUG_COLOR, window::UiWindow},
    kit::sonata::{Sonata, text::text::TextStyle},
};

mod vars;
mod widget;

fn custom_scrollable_style(_: &Theme, status: scrollable::Status) -> scrollable::Style {
    let scroller = Scroller {
        background: match status {
            scrollable::Status::Dragged { .. } => {
                Background::Color(Color::from_rgb8(185, 186, 189))
            } // When dragging
            scrollable::Status::Hovered { .. } => {
                Background::Color(Color::from_rgb8(207, 209, 210))
            } // On hover
            _ => Background::Color(Color::from_rgb8(207, 209, 210)), // Default
        },
        border: Border {
            color: Color::TRANSPARENT,
            width: 0.0,
            radius: 999.into(),
        },
    };

    let rail = Rail {
        background: match status {
            scrollable::Status::Hovered { .. } => {
                Some(Background::Color(Color::from_rgb8(230, 231, 232)))
            }
            scrollable::Status::Dragged { .. } => {
                Some(Background::Color(Color::from_rgb8(230, 231, 232)))
            }
            _ => Some(Background::Color(Color::TRANSPARENT)),
        },
        border: Border {
            color: Color::TRANSPARENT,
            width: 0.0,
            radius: 999.into(),
        },
        scroller,
    };

    let auto_scroll = AutoScroll {
        background: Background::Color(Color::from_rgba(0.0, 0.0, 0.0, 0.5)),
        border: Border {
            color: Color::BLACK,
            width: 1.0,
            radius: 5.0.into(),
        },
        shadow: Shadow::default(),
        icon: Color::WHITE,
    };

    scrollable::Style {
        container: iced::widget::container::Style::default(), // Background of the scroll area
        vertical_rail: rail,
        horizontal_rail: rail,
        gap: None,
        auto_scroll,
    }
}

impl<Message: Clone + 'static> Sonata<Message> {
    pub fn window<'a>(&self, params: UiWindow<'a, Message>) -> Element<'a, Message> {
        let header = container(
            text(params.label)
                .size(18)
                .font(TextStyle::TextLgSm.build_font()),
        )
        .style(|_| container::Style {
            background: Some(Background::Color(Color::TRANSPARENT)),
            border: *vars::HEADER_BORDER,
            shadow: *vars::HEADER_BOTTOM_SHADOW,
            ..Default::default()
        })
        .width(Fill)
        .padding(*vars::HEADER_PADDING);

        let content = container(
            scrollable(
                container(column![
                    params.child,
                    if params.controls_child.is_some() {
                        space().height(64)
                    } else {
                        space().height(0)
                    },
                    space().height(100)
                ])
                .width(Fill),
            )
            .style(custom_scrollable_style)
            .direction(Direction::Vertical(
                Scrollbar::new().width(5).scroller_width(5),
            )),
        )
        .style(|_| container::Style {
            background: None,
            border: Border {
                color: Color::TRANSPARENT,
                width: 0.0,
                radius: Radius::from(0.0),
            },
            ..Default::default()
        })
        .height(Shrink)
        .width(Fill)
        .padding(Padding {
            top: 5.0,
            left: 0.0,
            right: 5.0,
            bottom: 5.0,
        });

        let main_content = column![header, content].width(Fill);

        let controls_overlay = if let Some(controls) = params.controls_child {
            container(
                container(controls)
                    .style(|_| container::Style {
                        background: *vars::BACKGROUND,
                        border: Border {
                            color: Color::TRANSPARENT,
                            width: 0.0,
                            radius: *vars::CONTROLS_RADIUS,
                        },
                        ..Default::default()
                    })
                    .height(64)
                    .width(Fill)
                    .padding(*vars::CONTROLS_PADDING),
            )
            .align_x(Horizontal::Center)
            .align_y(Vertical::Bottom)
            .width(Fill)
            .height(Fill)
        } else {
            container(Space::new()).width(Fill).height(Fill)
        };

        let window_content = stack![main_content, controls_overlay];

        let window = container(window_content)
            .style(|_| container::Style {
                background: *vars::BACKGROUND,
                border: Border {
                    color: Color::TRANSPARENT,
                    width: 0.0,
                    radius: Radius::from(25),
                },
                ..Default::default()
            })
            .height(Shrink)
            .width(400)
            .clip(true);

        let window_wrapper = stack![if let Some(close_event) = params.wrapper_close_event {
            container(opaque(
                mouse_area(
                    center(opaque(window))
                        .style(|_| container::Style {
                            background: Some(
                                Color {
                                    a: 0.4,
                                    ..Color::BLACK
                                }
                                .into(),
                            ),
                            ..Default::default()
                        })
                        .padding(15),
                )
                .on_press(close_event),
            ))
        } else {
            window
        }];

        window_wrapper.into()
    }
}
