use iced::{
    Background, Border, Color, Element, Length, Padding, Shadow, Size, Theme, Vector,
    advanced::{
        Clipboard, Shell, Widget,
        graphics::core,
        renderer::Quad,
        widget::{Tree, tree},
    },
    alignment,
    border::Radius,
    widget::{self, container::layout, text_input},
};

pub struct FocusableInputWrapper<'a, Message, Theme, Renderer>
where
    Theme: Catalog,
    Renderer: core::Renderer,
{
    id: Option<widget::Id>,
    padding: Padding,
    width: Length,
    height: Length,
    max_width: f32,
    max_height: f32,
    horizontal_alignment: alignment::Horizontal,
    vertical_alignment: alignment::Vertical,
    clip: bool,
    content: Element<'a, Message, Theme, Renderer>,
    class: Theme::Class<'a>,
}

#[derive(Default)]
struct State {
    focused: bool,
}

impl<'a, Message, Theme, Renderer> FocusableInputWrapper<'a, Message, Theme, Renderer>
where
    Theme: Catalog,
    Renderer: core::Renderer,
{
    pub fn new(content: impl Into<Element<'a, Message, Theme, Renderer>>) -> Self {
        let content = content.into();
        let size = content.as_widget().size_hint();

        Self {
            id: None,
            padding: Padding::ZERO,
            width: size.width.fluid(),
            height: size.height.fluid(),
            max_width: f32::INFINITY,
            max_height: f32::INFINITY,
            horizontal_alignment: alignment::Horizontal::Left,
            vertical_alignment: alignment::Vertical::Top,
            clip: false,
            content: content,
            class: Theme::default(),
        }
    }

    pub fn style(mut self, style: impl Fn(&Theme) -> Style + 'a) -> Self
    where
        Theme::Class<'a>: From<StyleFn<'a, Theme>>,
    {
        self.class = (Box::new(style) as StyleFn<'a, Theme>).into();
        self
    }
}

impl<'a, Message, Theme, Renderer> Widget<Message, Theme, Renderer>
    for FocusableInputWrapper<'a, Message, Theme, Renderer>
where
    Theme: Catalog,
    Renderer: core::Renderer + iced::advanced::text::Renderer,
{
    fn tag(&self) -> tree::Tag {
        tree::Tag::of::<State>()
    }

    fn state(&self) -> core::widget::tree::State {
        tree::State::new(State::default())
    }

    fn children(&self) -> Vec<Tree> {
        vec![Tree::new(self.content.as_widget())]
    }

    fn diff(&self, tree: &mut Tree) {
        // self.content.as_widget().diff(tree);
        tree.diff_children(&[self.content.as_widget()]);
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
        if tree.children.is_empty() {
            tree.children.push(Tree::new(self.content.as_widget()));
        }

        layout(
            limits,
            self.width.fluid(),
            self.height.fluid(),
            self.max_width,
            self.max_height,
            self.padding,
            self.horizontal_alignment,
            self.vertical_alignment,
            |limits| {
                self.content
                    .as_widget_mut()
                    .layout(&mut tree.children[0], renderer, limits)
            },
        )
    }

    fn operate(
        &mut self,
        tree: &mut Tree,
        layout: core::Layout<'_>,
        renderer: &Renderer,
        operation: &mut dyn core::widget::Operation,
    ) {
        operation.container(self.id.as_ref(), layout.bounds());
        operation.traverse(&mut |operation| {
            self.content.as_widget_mut().operate(
                &mut tree.children[0],
                layout.children().next().unwrap(),
                renderer,
                operation,
            );
        });
    }

    fn update(
        &mut self,
        tree: &mut Tree,
        event: &iced::Event,
        layout: core::Layout<'_>,
        cursor: core::mouse::Cursor,
        renderer: &Renderer,
        clipboard: &mut dyn Clipboard,
        shell: &mut Shell<'_, Message>,
        viewport: &iced::Rectangle,
    ) {
        let child_layout = layout.children().next().unwrap();

        let child_state = tree.children[0]
            .state
            .downcast_ref::<text_input::State<Renderer::Paragraph>>();

        let state = tree.state.downcast_mut::<State>();

        state.focused = child_state.is_focused();

        self.content.as_widget_mut().update(
            &mut tree.children[0],
            event,
            child_layout,
            cursor,
            renderer,
            clipboard,
            shell,
            viewport,
        );
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
        let state = tree.state.downcast_ref::<State>();
        let bounds = layout.bounds();

        if let Some(clipped_viewport) = bounds.intersection(viewport) {
            if state.focused {
                renderer.fill_quad(
                    Quad {
                        bounds: layout.bounds(),
                        border: Border {
                            radius: Radius::from(10.0),
                            width: 2.0,
                            color: Color::from_rgba8(0, 122, 255, 0.5),
                        },
                        shadow: Shadow::default(),
                        snap: false,
                    },
                    Background::Color(Color::TRANSPARENT),
                );
            }

            self.content.as_widget().draw(
                &tree.children[0],
                renderer,
                theme,
                style,
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
        layout: core::Layout<'b>,
        renderer: &Renderer,
        viewport: &iced::Rectangle,
        translation: Vector,
    ) -> Option<core::overlay::Element<'b, Message, Theme, Renderer>> {
        self.content.as_widget_mut().overlay(
            &mut tree.children[0],
            layout.children().next().unwrap(),
            renderer,
            viewport,
            translation,
        )
    }

    fn mouse_interaction(
        &self,
        tree: &Tree,
        layout: core::Layout<'_>,
        cursor: core::mouse::Cursor,
        viewport: &iced::Rectangle,
        renderer: &Renderer,
    ) -> core::mouse::Interaction {
        self.content.as_widget().mouse_interaction(
            &tree.children[0],
            layout.children().next().unwrap(),
            cursor,
            viewport,
            renderer,
        )
    }
}

pub struct Style {
    pub shadow: Shadow,
}

impl Default for Style {
    fn default() -> Self {
        Self {
            shadow: Shadow {
                color: Color::BLACK,
                offset: Vector::ZERO,
                blur_radius: 10.0,
            },
        }
    }
}

pub trait Catalog {
    type Class<'a>;

    fn default<'a>() -> Self::Class<'a>;

    fn style(&self, class: &Self::Class<'_>) -> Style;
}

pub type StyleFn<'a, Theme> = Box<dyn Fn(&Theme) -> Style + 'a>;

impl Catalog for Theme {
    type Class<'a> = StyleFn<'a, Self>;

    fn default<'a>() -> Self::Class<'a> {
        Box::new(transparent)
    }

    fn style(&self, class: &Self::Class<'_>) -> Style {
        class(self)
    }
}

fn transparent<Theme>(_theme: &Theme) -> Style {
    Style::default()
}

impl<'a, Message, Theme, Renderer> From<FocusableInputWrapper<'a, Message, Theme, Renderer>>
    for Element<'a, Message, Theme, Renderer>
where
    Message: 'a,
    Theme: Catalog + 'a,
    Renderer: core::Renderer + 'a + iced::advanced::text::Renderer,
{
    fn from(
        value: FocusableInputWrapper<'a, Message, Theme, Renderer>,
    ) -> Element<'a, Message, Theme, Renderer> {
        Element::new(value)
    }
}

pub fn focusable_wrapper<'a, Message, Theme, Renderer>(
    el: Element<'a, Message, Theme, Renderer>,
) -> FocusableInputWrapper<'a, Message, Theme, Renderer>
where
    Message: Clone,
    Theme: Catalog + 'a,
    Renderer: core::Renderer + 'a + iced::advanced::text::Renderer,
{
    FocusableInputWrapper::new(el)
}
