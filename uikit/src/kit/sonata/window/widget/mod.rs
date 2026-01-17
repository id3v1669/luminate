use ::core::f32;

use iced::{
    Background, Border, Color, Element, Length, Padding, Rectangle, Shadow, Size,
    advanced::{
        Widget,
        graphics::core::{self},
        renderer,
    },
    alignment,
    widget::{self, container::layout},
};
use style::Catalog;

use crate::kit::sonata::window::widget::style::Style;

mod style;

pub struct Window<'a, Message, Theme = iced::Theme, Renderer = iced::Renderer>
where
    Theme: Catalog,
    Renderer: core::Renderer,
{
    id: Option<widget::Id>,
    width: Length,
    height: Length,
    padding: Padding,
    max_width: f32,
    max_height: f32,
    clip: bool,
    horizontal_alignment: alignment::Horizontal,
    vertical_alignment: alignment::Vertical,
    class: Theme::Class<'a>,

    content: Element<'a, Message, Theme, Renderer>,
    controls_content: Option<Element<'a, Message, Theme, Renderer>>,
}

impl<'a, Message, Theme, Renderer> Window<'a, Message, Theme, Renderer>
where
    Theme: Catalog,
    Renderer: core::Renderer,
{
    pub fn new(content: impl Into<Element<'a, Message, Theme, Renderer>>) -> Self {
        let content = content.into();
        let size = content.as_widget().size_hint();

        Self {
            id: None,
            width: size.width.fluid(),
            height: size.height.fluid(),
            padding: Padding::ZERO,
            max_width: f32::INFINITY,
            max_height: f32::INFINITY,
            clip: false,
            horizontal_alignment: alignment::Horizontal::Center,
            vertical_alignment: alignment::Vertical::Center,
            class: Theme::default(),

            content,
            controls_content: None,
        }
    }
}

impl<'a, Message, Theme, Renderer> Widget<Message, Theme, Renderer>
    for Window<'a, Message, Theme, Renderer>
where
    Theme: Catalog,
    Renderer: core::Renderer,
{
    fn size(&self) -> iced::Size<Length> {
        Size {
            width: self.width,
            height: self.height,
        }
    }

    fn layout(
        &mut self,
        tree: &mut core::widget::Tree,
        renderer: &Renderer,
        limits: &core::layout::Limits,
    ) -> core::layout::Node {
        layout(
            limits,
            self.width,
            self.height,
            self.max_width,
            self.max_height,
            self.padding,
            self.horizontal_alignment,
            self.vertical_alignment,
            |limits| self.content.as_widget_mut().layout(tree, renderer, limits),
        )
    }

    fn draw(
        &self,
        tree: &core::widget::Tree,
        renderer: &mut Renderer,
        theme: &Theme,
        style: &core::renderer::Style,
        layout: core::Layout<'_>,
        cursor: core::mouse::Cursor,
        viewport: &iced::Rectangle,
    ) {
        let bounds = layout.bounds();
        let widget_style = theme.style(&self.class);

        if let Some(clipped_viewport) = bounds.intersection(viewport) {
            draw_background(renderer, &widget_style, bounds);

            self.content.as_widget().draw(
                tree,
                renderer,
                theme,
                &renderer::Style {
                    text_color: widget_style.text_color.unwrap_or(style.text_color),
                },
                layout.children().next().unwrap(),
                cursor,
                if self.clip {
                    &clipped_viewport
                } else {
                    viewport
                },
            );
        }
    }
}

pub fn draw_background<Renderer>(renderer: &mut Renderer, style: &Style, bounds: Rectangle)
where
    Renderer: core::Renderer,
{
    if style.background.is_some() {
        renderer.fill_quad(
            renderer::Quad {
                bounds,
                border: Border::default(),
                shadow: Shadow::default(),
                snap: true,
            },
            style
                .background
                .unwrap_or(Background::Color(Color::TRANSPARENT)),
        );
    }
}
