use ::core::f32;
use std::slice;

use iced::{
    Background, Border, Color, Element, Event, Length, Padding, Rectangle, Renderer, Shadow, Size,
    Theme, Vector,
    advanced::{
        Clipboard, Layout, Shell, Widget,
        graphics::core,
        mouse, overlay, renderer,
        widget::{Operation, Tree, tree},
    },
    alignment,
    widget::{self, container::layout},
};
use style::Catalog;

use crate::{definition::window::UiWindow, kit::sonata::components::window::widget::style::Style};

mod style;

pub struct Window<'a, Message, Theme = iced::Theme>
where
    Theme: Catalog,
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

    params: UiWindow<'a, Message>,
}

#[derive(Debug, Default)]
struct State {
    toggle: bool,
}

impl<'a, Message, Theme> Window<'a, Message, Theme>
where
    Theme: Catalog,
{
    pub fn new(params: UiWindow<'a, Message>) -> Self {
        let size = params.child.as_widget().size_hint();

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

            params,
        }
    }
}

impl<'a, Message> Widget<Message, Theme, Renderer> for Window<'a, Message, Theme>
where
    Theme: Catalog,
    Renderer: renderer::Renderer,
{
    fn tag(&self) -> core::widget::tree::Tag {
        tree::Tag::of::<State>()
    }

    fn state(&self) -> core::widget::tree::State {
        // self.params.child.as_widget().state()
        core::widget::tree::State::new(State::default())
    }

    fn children(&self) -> Vec<tree::Tree> {
        // self.params.child.as_widget().children()
        vec![Tree::new(self.params.child.as_widget())]
    }

    fn diff(&self, tree: &mut tree::Tree) {
        // if tree.children.is_empty() {
        //     tree.children.push(Tree::new(self.params.child.as_widget()));
        // }

        // // Call child.diff on the child's Tree (not on the parent Tree).
        // self.params.child.as_widget().diff(&mut tree.children[0]);

        tree.diff_children(slice::from_ref(&self.params.child));
    }

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
        // let width = limits.max().width;
        // let header_height = 50.0;
        // let content_limits = limits.clone().height(limits.max().height - header_height);

        // let child_node = self.params.child.as_widget_mut().layout(
        //     &mut tree.children[0],
        //     renderer,
        //     &content_limits,
        // );

        if tree.children.is_empty() {
            tree.children.push(Tree::new(self.params.child.as_widget()));
        }

        layout(
            limits,
            self.width,
            self.height,
            self.max_width,
            self.max_height,
            self.padding,
            self.horizontal_alignment,
            self.vertical_alignment,
            |limits| {
                self.params
                    .child
                    .as_widget_mut()
                    .layout(&mut tree.children[0], renderer, limits)
            },
        )
    }

    fn operate(
        &mut self,
        tree: &mut Tree,
        layout: Layout<'_>,
        renderer: &Renderer,
        operation: &mut dyn Operation,
    ) {
        operation.container(self.id.as_ref(), layout.bounds());
        operation.traverse(&mut |operation| {
            self.params.child.as_widget_mut().operate(
                tree,
                layout.children().next().unwrap(),
                renderer,
                operation,
            );
        });
    }

    fn update(
        &mut self,
        tree: &mut Tree,
        event: &Event,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        renderer: &Renderer,
        clipboard: &mut dyn Clipboard,
        shell: &mut Shell<'_, Message>,
        viewport: &Rectangle,
    ) {
        self.params.child.as_widget_mut().update(
            tree,
            event,
            layout.children().next().unwrap(),
            cursor,
            renderer,
            clipboard,
            shell,
            viewport,
        );
    }

    fn mouse_interaction(
        &self,
        tree: &Tree,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        viewport: &Rectangle,
        renderer: &Renderer,
    ) -> mouse::Interaction {
        self.params.child.as_widget().mouse_interaction(
            tree,
            layout.children().next().unwrap(),
            cursor,
            viewport,
            renderer,
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

            self.params.child.as_widget().draw(
                &tree.children[0],
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

    fn overlay<'b>(
        &'b mut self,
        tree: &'b mut Tree,
        layout: Layout<'b>,
        renderer: &Renderer,
        viewport: &Rectangle,
        translation: Vector,
    ) -> Option<overlay::Element<'b, Message, Theme, Renderer>> {
        self.params.child.as_widget_mut().overlay(
            tree,
            layout.children().next().unwrap(),
            renderer,
            viewport,
            translation,
        )
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

impl<'a, Message> From<Window<'a, Message>> for Element<'a, Message> {
    fn from(widget: Window<'a, Message>) -> Self {
        Element::new(widget)
    }
}
