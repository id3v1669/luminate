//! A small speech bubble anchored to a child, for validation messages.

use std::f32::consts::PI;
use std::sync::LazyLock;

use iced::advanced::layout::{Limits, Node};
use iced::advanced::renderer::Quad;
use iced::advanced::text::{self, Fragment, IntoFragment, LineHeight, Paragraph, paragraph::Plain};
use iced::advanced::widget::{self, Tree, tree};
use iced::advanced::{Clipboard, Layout, Shell, Widget, mouse, overlay, renderer, svg};
use iced::border::Radius;
use iced::widget::text::{Alignment, Shaping, Wrapping};
use iced::{Color, Event, Font, Length, Padding, Pixels, Point, Radians, Rectangle, Size, Vector};

/// What an [`ErrorBubble`] is painted with.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Style {
    /// Fill of the bubble and arrow.
    pub background: Color,
    /// Message text colour.
    pub text: Color,
    /// Corner radius of the bubble.
    pub radius: f32,
}

catalog!(|theme| {
    let palette = theme.extended_palette();

    Style {
        background: palette.danger.weak.color,
        text: palette.danger.weak.text,
        radius: 10.0,
    }
});

/// Font, size and line height of the message; `None` means the renderer's
/// default.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
struct Typography {
    font: Option<Font>,
    size: Option<Pixels>,
    line_height: LineHeight,
}

/// The arrow, tinted with the bubble colour at draw time. Its tip is at the
/// centre of the view box so a half-turn for [`Placement::Below`] keeps it
/// under the same x.
const ARROW_SVG: &[u8] = br#"<svg width="44" height="22" viewBox="0 0 44 22" preserveAspectRatio="none" xmlns="http://www.w3.org/2000/svg"><path d="M0,0 C8,0 14,6 18,10 L20,14 C21,16 22,18 22,18 C22,18 23,16 24,14 L26,10 C30,6 36,0 44,0" fill="currentColor"/></svg>"#;

static ARROW: LazyLock<svg::Handle> = LazyLock::new(|| svg::Handle::from_memory(ARROW_SVG));

/// The corner the arrow hangs from is tightened by this factor so the
/// arrow's base meets a flatter edge.
const ARROW_CORNER_FACTOR: f32 = 0.6;

/// Sizes an [`ErrorBubble`] is laid out with; set through its builders.
#[derive(Debug, Clone, Copy, PartialEq)]
struct Metrics {
    /// Padding around the message.
    padding: Padding,
    /// Space between the arrow tip and the child.
    gap: f32,
    /// Size of the arrow.
    arrow: Size,
    /// Distance from the bubble's right edge to the arrow's right edge.
    arrow_offset: f32,
    /// Distance from the child's right edge to the bubble's right edge.
    right_offset: f32,
    /// Widest the bubble grows before the message wraps.
    max_width: f32,
}

impl Metrics {
    const DEFAULT: Self = Self {
        padding: Padding {
            top: 6.0,
            right: 10.0,
            bottom: 6.0,
            left: 10.0,
        },
        gap: 5.5,
        arrow: Size::new(11.0, 5.5),
        arrow_offset: 11.0,
        right_offset: 7.0,
        max_width: 320.0,
    };
}

/// Where the bubble sits relative to its anchor.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Placement {
    Above,
    Below,
}

/// Bubble geometry, in window space.
#[derive(Debug, Clone, Copy, PartialEq)]
struct Geometry {
    bubble: Rectangle,
    arrow: Rectangle,
    placement: Placement,
}

/// Places a `bubble_size` bubble against `anchor` inside `viewport`.
///
/// Above the anchor when it fits, else below, else pinned to the edge of
/// whichever side has more room.
fn place(anchor: Rectangle, bubble_size: Size, viewport: Size, metrics: &Metrics) -> Geometry {
    let clearance = (metrics.arrow.height + metrics.gap).round();
    let space_above = anchor.y;
    let space_below = viewport.height - (anchor.y + anchor.height);

    let (y, placement) = if space_above >= bubble_size.height + clearance {
        (anchor.y - bubble_size.height - clearance, Placement::Above)
    } else if space_below >= bubble_size.height + clearance {
        (anchor.y + anchor.height + clearance, Placement::Below)
    } else if space_above >= space_below {
        (0.0, Placement::Above)
    } else {
        (viewport.height - bubble_size.height, Placement::Below)
    };

    let x = (anchor.x + anchor.width - bubble_size.width - metrics.right_offset)
        .max(0.0)
        .min((viewport.width - bubble_size.width).max(0.0));

    let bubble = Rectangle::new(Point::new(x, y), bubble_size);

    let arrow_size = Size::new(metrics.arrow.width.round(), metrics.arrow.height.round());
    let arrow_x = bubble.x + bubble.width - arrow_size.width - metrics.arrow_offset;
    let arrow_y = match placement {
        Placement::Above => bubble.y + bubble.height,
        Placement::Below => bubble.y - arrow_size.height,
    };

    Geometry {
        bubble,
        arrow: Rectangle::new(Point::new(arrow_x, arrow_y), arrow_size),
        placement,
    }
}

/// Wraps `child` and, while it has a message, floats it in a bubble above
/// the child (below when there is no room above) as an overlay.
///
/// The child's own overlay (a pick-list menu, a tooltip, …) stays: both are
/// grouped and the bubble draws on top. The message wraps at words up to
/// [`max_width`](Self::max_width). Colours come from the [`Catalog`]; every
/// size has a builder with a neutral default.
///
/// # Example
///
/// ```
/// use iced_luminate::iced::widget::text_input;
/// use iced_luminate::iced::{Element, Theme};
/// use iced_luminate::widget::error_bubble::error_bubble;
///
/// #[derive(Clone)]
/// enum Message {
///     Typed(String),
/// }
///
/// let value = "";
/// let error: Option<&str> = Some("Required");
/// let field: Element<'_, Message, Theme, iced_luminate::Renderer> =
///     error_bubble(text_input("Name", value).on_input(Message::Typed), error)
///         .padding([6.0, 10.0])
///         .gap(4.0)
///         .into();
/// ```
pub struct ErrorBubble<'a, Message, Theme = iced::Theme, Renderer = crate::Renderer>
where
    Theme: Catalog,
{
    child: iced::Element<'a, Message, Theme, Renderer>,
    /// `None` hides the bubble.
    message: Option<Fragment<'a>>,
    metrics: Metrics,
    typography: Typography,
    class: Theme::Class<'a>,
}

impl<Message, Theme, Renderer> std::fmt::Debug for ErrorBubble<'_, Message, Theme, Renderer>
where
    Theme: Catalog,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ErrorBubble")
            .field("message", &self.message)
            .field("metrics", &self.metrics)
            .field("typography", &self.typography)
            .finish_non_exhaustive()
    }
}

/// Widget state: the shaped message and the geometry of the last overlay
/// layout.
///
/// iced builds one overlay instance for `update` (which lays it out) and a
/// fresh one for `draw` (which is not laid out again: the node from
/// `update` is reused), so anything `draw` needs from `layout` must live
/// here, not on the overlay.
struct State<P: Paragraph> {
    paragraph: Plain<P>,
    geometry: Option<Geometry>,
}

impl<'a, Message, Theme, Renderer> ErrorBubble<'a, Message, Theme, Renderer>
where
    Theme: Catalog,
    Renderer: renderer::Renderer,
{
    /// Wraps `child`; shows `message` in a bubble while it is `Some`.
    ///
    /// `message` is generic over the text type, so a call without a
    /// message must name one: `ErrorBubble::new(child, None::<&str>)`.
    #[must_use]
    pub fn new(
        child: impl Into<iced::Element<'a, Message, Theme, Renderer>>,
        message: Option<impl IntoFragment<'a>>,
    ) -> Self {
        Self {
            child: child.into(),
            message: message.map(IntoFragment::into_fragment),
            metrics: Metrics::DEFAULT,
            typography: Typography::default(),
            class: Theme::default(),
        }
    }

    /// Padding around the message (default 6 vertical, 10 horizontal).
    #[must_use]
    pub fn padding(mut self, padding: impl Into<Padding>) -> Self {
        self.metrics.padding = padding.into();
        self
    }

    /// Space between the arrow tip and the child (default 5.5).
    #[must_use]
    pub fn gap(mut self, gap: impl Into<Pixels>) -> Self {
        self.metrics.gap = gap.into().0;
        self
    }

    /// Size of the arrow (default 11 × 5.5).
    #[must_use]
    pub fn arrow(mut self, width: impl Into<Pixels>, height: impl Into<Pixels>) -> Self {
        self.metrics.arrow = Size::new(width.into().0, height.into().0);
        self
    }

    /// Distance from the bubble's right edge to the arrow's right edge
    /// (default 11).
    #[must_use]
    pub fn arrow_offset(mut self, offset: impl Into<Pixels>) -> Self {
        self.metrics.arrow_offset = offset.into().0;
        self
    }

    /// Distance from the child's right edge to the bubble's right edge
    /// (default 7).
    #[must_use]
    pub fn right_offset(mut self, offset: impl Into<Pixels>) -> Self {
        self.metrics.right_offset = offset.into().0;
        self
    }

    /// Widest the bubble grows before the message wraps (default 320).
    #[must_use]
    pub fn max_width(mut self, max_width: impl Into<Pixels>) -> Self {
        self.metrics.max_width = max_width.into().0;
        self
    }

    /// Font, size and line height of the message (default: the renderer's).
    #[must_use]
    pub fn text_style(
        mut self,
        font: Font,
        size: impl Into<Pixels>,
        line_height: impl Into<LineHeight>,
    ) -> Self {
        self.typography = Typography {
            font: Some(font),
            size: Some(size.into()),
            line_height: line_height.into(),
        };
        self
    }

    /// Sets the style with a closure.
    #[must_use]
    pub fn style(mut self, style: impl Fn(&Theme) -> Style + 'a) -> Self
    where
        Theme::Class<'a>: From<StyleFn<'a, Theme>>,
    {
        self.class = (Box::new(style) as StyleFn<'a, Theme>).into();
        self
    }

    /// Uses a class from the theme's catalog.
    #[must_use]
    pub fn class(mut self, class: impl Into<Theme::Class<'a>>) -> Self {
        self.class = class.into();
        self
    }
}

/// The single child of an [`ErrorBubble`] layout.
fn child_layout(layout: Layout<'_>) -> Layout<'_> {
    layout
        .children()
        .next()
        .expect("ErrorBubble layout has exactly one child")
}

impl<Message, Theme, Renderer> Widget<Message, Theme, Renderer>
    for ErrorBubble<'_, Message, Theme, Renderer>
where
    Theme: Catalog,
    Renderer: text::Renderer<Font = Font> + svg::Renderer,
{
    fn size(&self) -> Size<Length> {
        self.child.as_widget().size()
    }

    fn tag(&self) -> tree::Tag {
        tree::Tag::of::<State<Renderer::Paragraph>>()
    }

    fn state(&self) -> tree::State {
        tree::State::new(State::<Renderer::Paragraph> {
            paragraph: Plain::default(),
            geometry: None,
        })
    }

    fn children(&self) -> Vec<Tree> {
        vec![Tree::new(&self.child)]
    }

    fn diff(&self, tree: &mut Tree) {
        tree.diff_children(std::slice::from_ref(&self.child));
    }

    fn layout(&mut self, tree: &mut Tree, renderer: &Renderer, limits: &Limits) -> Node {
        let child = self
            .child
            .as_widget_mut()
            .layout(&mut tree.children[0], renderer, limits);

        Node::with_children(child.size(), vec![child])
    }

    fn draw(
        &self,
        tree: &Tree,
        renderer: &mut Renderer,
        theme: &Theme,
        style: &renderer::Style,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        viewport: &Rectangle,
    ) {
        self.child.as_widget().draw(
            &tree.children[0],
            renderer,
            theme,
            style,
            child_layout(layout),
            cursor,
            viewport,
        );
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
        self.child.as_widget_mut().update(
            &mut tree.children[0],
            event,
            child_layout(layout),
            cursor,
            renderer,
            clipboard,
            shell,
            viewport,
        );
    }

    fn operate(
        &mut self,
        tree: &mut Tree,
        layout: Layout<'_>,
        renderer: &Renderer,
        operation: &mut dyn widget::Operation,
    ) {
        self.child.as_widget_mut().operate(
            &mut tree.children[0],
            child_layout(layout),
            renderer,
            operation,
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
        self.child.as_widget().mouse_interaction(
            &tree.children[0],
            child_layout(layout),
            cursor,
            viewport,
            renderer,
        )
    }

    fn overlay<'b>(
        &'b mut self,
        tree: &'b mut Tree,
        layout: Layout<'b>,
        renderer: &Renderer,
        viewport: &Rectangle,
        translation: Vector,
    ) -> Option<overlay::Element<'b, Message, Theme, Renderer>> {
        // `state` and `children` are disjoint borrows of the tree: the child
        // keeps its subtree, the bubble its paragraph.
        let Tree {
            state, children, ..
        } = tree;

        let child_overlay = self.child.as_widget_mut().overlay(
            &mut children[0],
            child_layout(layout),
            renderer,
            viewport,
            translation,
        );

        let Some(text) = self.message.as_deref() else {
            return child_overlay;
        };

        let bubble = overlay::Element::new(Box::new(Bubble {
            anchor: layout.bounds() + translation,
            text,
            metrics: self.metrics,
            typography: self.typography,
            state: state.downcast_mut::<State<Renderer::Paragraph>>(),
            class: &self.class,
        }));

        Some(match child_overlay {
            Some(child) => overlay::Group::with_children(vec![child, bubble]).overlay(),
            None => bubble,
        })
    }
}

struct Bubble<'a, 'b, Theme: Catalog, P: Paragraph> {
    anchor: Rectangle,
    text: &'b str,
    metrics: Metrics,
    typography: Typography,
    /// Shared with the widget: `layout` stores the geometry here and the
    /// instance iced builds for `draw` reads it back.
    state: &'b mut State<P>,
    class: &'b Theme::Class<'a>,
}

impl<Message, Theme, Renderer> overlay::Overlay<Message, Theme, Renderer>
    for Bubble<'_, '_, Theme, Renderer::Paragraph>
where
    Theme: Catalog,
    Renderer: text::Renderer<Font = Font> + svg::Renderer,
{
    fn layout(&mut self, renderer: &Renderer, bounds: Size) -> Node {
        // The cap applies to the bubble, so the padding comes off after it.
        let limits = Limits::new(Size::ZERO, bounds)
            .max_width(self.metrics.max_width)
            .shrink(self.metrics.padding);

        widget::text::layout(
            &mut self.state.paragraph,
            renderer,
            &limits,
            self.text,
            widget::text::Format {
                width: Length::Shrink,
                height: Length::Shrink,
                size: self.typography.size,
                font: self.typography.font,
                line_height: self.typography.line_height,
                align_x: Alignment::Default,
                align_y: iced::alignment::Vertical::Top,
                shaping: Shaping::Advanced,
                wrapping: Wrapping::Word,
            },
        );

        let text_size = self.state.paragraph.raw().min_bounds();
        let bubble_size = Size::new(
            text_size.width + self.metrics.padding.x(),
            text_size.height + self.metrics.padding.y(),
        );

        self.state.geometry = Some(place(self.anchor, bubble_size, bounds, &self.metrics));

        // iced clips an overlay to its node, and the arrow hangs outside the
        // bubble: the node covers the whole viewport (as a `Group` does) and
        // `draw` takes the bubble rectangle from the stored geometry. The
        // bubble never reports a mouse interaction, so it captures nothing.
        Node::new(bounds)
    }

    fn draw(
        &self,
        renderer: &mut Renderer,
        theme: &Theme,
        style: &renderer::Style,
        _layout: Layout<'_>,
        _cursor: mouse::Cursor,
    ) {
        // Nothing to draw until the update-time instance has laid out.
        let Some(Geometry {
            bubble,
            arrow,
            placement,
        }) = self.state.geometry
        else {
            return;
        };
        let colors = theme.style(self.class);
        let r = colors.radius;
        let corner = r * ARROW_CORNER_FACTOR;

        let (radius, rotation) = match placement {
            Placement::Above => (
                Radius {
                    top_left: r,
                    top_right: r,
                    bottom_left: r,
                    bottom_right: corner,
                },
                0.0,
            ),
            Placement::Below => (
                Radius {
                    top_left: r,
                    top_right: corner,
                    bottom_left: r,
                    bottom_right: r,
                },
                PI,
            ),
        };

        renderer.fill_quad(
            Quad {
                bounds: bubble,
                border: iced::Border {
                    radius,
                    width: 0.0,
                    color: Color::TRANSPARENT,
                },
                ..Default::default()
            },
            colors.background,
        );

        renderer.draw_svg(
            svg::Svg {
                handle: ARROW.clone(),
                color: Some(colors.background),
                rotation: Radians(rotation),
                opacity: 1.0,
            },
            arrow,
            arrow.expand(1.0),
        );

        let padding = self.metrics.padding;
        let text_bounds = Rectangle {
            x: bubble.x + padding.left,
            y: bubble.y + padding.top,
            width: bubble.width - padding.x(),
            height: bubble.height - padding.y(),
        };
        widget::text::draw(
            renderer,
            style,
            text_bounds,
            self.state.paragraph.raw(),
            widget::text::Style {
                color: Some(colors.text),
            },
            &bubble,
        );
    }

    /// Above the child's own overlay in the [`overlay::Group`].
    fn index(&self) -> f32 {
        2.0
    }
}

impl<'a, Message, Theme, Renderer> From<ErrorBubble<'a, Message, Theme, Renderer>>
    for iced::Element<'a, Message, Theme, Renderer>
where
    Message: 'a,
    Theme: Catalog + 'a,
    Renderer: text::Renderer<Font = Font> + svg::Renderer + 'a,
{
    fn from(value: ErrorBubble<'a, Message, Theme, Renderer>) -> Self {
        iced::Element::new(value)
    }
}

/// An [`ErrorBubble`] around `child`, showing `message` while it is `Some`.
#[must_use]
pub fn error_bubble<'a, Message, Theme, Renderer>(
    child: impl Into<iced::Element<'a, Message, Theme, Renderer>>,
    message: Option<impl IntoFragment<'a>>,
) -> ErrorBubble<'a, Message, Theme, Renderer>
where
    Theme: Catalog,
    Renderer: renderer::Renderer,
{
    ErrorBubble::new(child, message)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn metrics() -> Metrics {
        Metrics::DEFAULT
    }

    #[test]
    fn the_bubble_goes_above_when_there_is_room_and_below_otherwise() {
        let viewport = Size::new(400.0, 300.0);
        let bubble = Size::new(100.0, 30.0);
        let anchor = Rectangle::new(Point::new(50.0, 150.0), Size::new(200.0, 40.0));
        let above = place(anchor, bubble, viewport, &metrics());
        assert_eq!(above.placement, Placement::Above);
        assert!(above.bubble.y + above.bubble.height < anchor.y);
        assert_eq!(above.arrow.y, above.bubble.y + above.bubble.height);

        let top = Rectangle::new(Point::new(50.0, 5.0), Size::new(200.0, 40.0));
        let below = place(top, bubble, viewport, &metrics());
        assert_eq!(below.placement, Placement::Below);
        assert!(below.bubble.y > top.y + top.height);
        assert_eq!(below.arrow.y, below.bubble.y - below.arrow.height);
    }

    #[test]
    fn when_nothing_fits_the_bubble_hugs_the_larger_side() {
        let viewport = Size::new(400.0, 100.0);
        let bubble = Size::new(100.0, 60.0);

        // More room above (40) than below (20): pinned to the top edge.
        let low = Rectangle::new(Point::new(50.0, 40.0), Size::new(200.0, 40.0));
        let g = place(low, bubble, viewport, &metrics());
        assert_eq!(g.placement, Placement::Above);
        assert_eq!(g.bubble.y, 0.0);

        // More room below (60) than above (20): pinned to the bottom edge.
        let high = Rectangle::new(Point::new(50.0, 20.0), Size::new(200.0, 20.0));
        let g = place(high, bubble, viewport, &metrics());
        assert_eq!(g.placement, Placement::Below);
        assert_eq!(g.bubble.y, viewport.height - bubble.height);
    }

    #[test]
    fn the_bubble_stays_inside_the_viewport_horizontally() {
        let viewport = Size::new(120.0, 300.0);
        let wide = Size::new(200.0, 30.0);
        let anchor = Rectangle::new(Point::new(10.0, 150.0), Size::new(50.0, 40.0));
        let g = place(anchor, wide, viewport, &metrics());
        assert_eq!(g.bubble.x, 0.0, "left clamp");

        let narrow = Size::new(50.0, 30.0);
        let far_right = Rectangle::new(Point::new(100.0, 150.0), Size::new(80.0, 40.0));
        let g = place(far_right, narrow, viewport, &metrics());
        assert_eq!(g.bubble.x + g.bubble.width, viewport.width, "right clamp");
    }

    #[test]
    fn the_bubble_right_edge_is_inset_from_the_anchor_right_edge() {
        let viewport = Size::new(400.0, 300.0);
        let bubble = Size::new(100.0, 30.0);
        let anchor = Rectangle::new(Point::new(50.0, 150.0), Size::new(200.0, 40.0));
        let g = place(anchor, bubble, viewport, &metrics());
        assert_eq!(
            g.bubble.x + g.bubble.width,
            anchor.x + anchor.width - metrics().right_offset
        );
    }

    #[test]
    fn the_arrow_right_edge_sits_arrow_offset_from_the_bubble_right_edge() {
        let viewport = Size::new(400.0, 300.0);
        let bubble = Size::new(100.0, 30.0);
        let anchor = Rectangle::new(Point::new(50.0, 150.0), Size::new(200.0, 40.0));
        let g = place(anchor, bubble, viewport, &metrics());
        let m = metrics();
        assert_eq!(g.arrow.width, m.arrow.width.round());
        assert_eq!(
            g.arrow.x + g.arrow.width,
            g.bubble.x + g.bubble.width - m.arrow_offset
        );
    }

    #[test]
    fn the_arrow_is_left_right_symmetric() {
        // Mirror every x around the 22-unit centre of the 44-unit viewBox.
        let svg = std::str::from_utf8(ARROW_SVG).unwrap();
        let path = svg.split("d=\"").nth(1).unwrap().split('"').next().unwrap();
        let xs: Vec<f32> = path
            .split([' ', ','])
            .filter_map(|token| {
                token
                    .trim_start_matches(['M', 'C', 'L'])
                    .parse::<f32>()
                    .ok()
            })
            .step_by(2)
            .collect();
        let mirrored: Vec<f32> = xs.iter().rev().map(|x| 44.0 - x).collect();
        assert_eq!(xs, mirrored, "{path}");
    }

    /// Lays `root` out in a `size` window, draws it once with the light theme
    /// on a white ground and returns the RGBA pixels at scale 1.
    fn render(
        root: iced::Element<'_, (), iced::Theme, crate::Renderer>,
        size: Size,
    ) -> (Vec<u8>, usize) {
        use iced::advanced::clipboard;
        use iced::advanced::renderer::Headless;
        use iced::time::Instant;
        use iced::window;
        use iced_test::runtime::UserInterface;
        use iced_test::runtime::user_interface::Cache;

        let mut renderer =
            iced_test::futures::futures::executor::block_on(<crate::Renderer as Headless>::new(
                iced::Font::DEFAULT,
                iced::Pixels(16.0),
                Some("tiny-skia"),
            ))
            .expect("tiny_skia needs no GPU");
        let mut ui = UserInterface::build(root, size, Cache::default(), &mut renderer);
        let mut messages = Vec::new();
        // The overlay is laid out by `update`, as in a running application.
        let _ = ui.update(
            &[Event::Window(
                window::Event::RedrawRequested(Instant::now()),
            )],
            mouse::Cursor::Unavailable,
            &mut renderer,
            &mut clipboard::Null,
            &mut messages,
        );
        ui.draw(
            &mut renderer,
            &iced::Theme::Light,
            &renderer::Style {
                text_color: Color::BLACK,
            },
            mouse::Cursor::Unavailable,
        );
        let width = size.width as usize;
        let rgba = renderer.screenshot(
            Size::new(size.width as u32, size.height as u32),
            1.0,
            Color::WHITE,
        );
        (rgba, width)
    }

    fn pixel(rgba: &[u8], width: usize, x: usize, y: usize) -> [u8; 3] {
        let i = (y * width + x) * 4;
        [rgba[i], rgba[i + 1], rgba[i + 2]]
    }

    fn close(a: [u8; 3], b: [u8; 3]) -> bool {
        a.iter().zip(b).all(|(a, b)| a.abs_diff(b) <= 2)
    }

    #[test]
    fn the_arrow_is_painted_outside_the_bubble() {
        use iced::widget::{Space, container};

        // A 200 × 40 anchor at (0, 100): the bubble goes above it.
        let root: iced::Element<'_, (), iced::Theme, crate::Renderer> = container(error_bubble(
            Space::new().width(200.0).height(40.0),
            Some("required"),
        ))
        .padding(Padding {
            top: 100.0,
            ..Padding::ZERO
        })
        .into();
        let (rgba, width) = render(root, Size::new(300.0, 200.0));

        let m = Metrics::DEFAULT;
        let fill = {
            let [r, g, b, _] = <iced::Theme as Catalog>::style(
                &iced::Theme::Light,
                &<iced::Theme as Catalog>::default(),
            )
            .background
            .into_rgba8();
            [r, g, b]
        };
        // Bubble bottom edge = anchor top − round(arrow height + gap).
        let bubble_bottom = 100 - (m.arrow.height + m.gap).round() as usize;
        // Arrow centre x: bubble right (anchor right − right offset) − arrow
        // offset − half the arrow width.
        let arrow_x = (200.0 - m.right_offset - m.arrow_offset - m.arrow.width / 2.0) as usize;

        let in_bubble = pixel(&rgba, width, arrow_x, bubble_bottom - 2);
        let in_arrow = pixel(&rgba, width, arrow_x, bubble_bottom + 1);
        let in_gap = pixel(&rgba, width, arrow_x, 99);
        assert!(close(in_bubble, fill), "bubble: {in_bubble:?} vs {fill:?}");
        assert!(
            close(in_arrow, fill),
            "arrow below the bubble: {in_arrow:?} vs {fill:?}"
        );
        assert!(close(in_gap, [255, 255, 255]), "gap: {in_gap:?}");
    }

    #[test]
    fn the_child_overlay_survives_a_shown_bubble() {
        use iced::widget::{container, pick_list};
        use iced_test::Simulator;

        #[derive(Debug, Clone, PartialEq)]
        enum Message {
            Picked(&'static str),
        }

        // 100 px above the pick list: the bubble goes above, the menu below.
        let root: iced::Element<'_, Message, iced::Theme, crate::Renderer> =
            container(error_bubble(
                pick_list(["only option"], None::<&'static str>, Message::Picked),
                Some("required"),
            ))
            .padding(Padding {
                top: 100.0,
                ..Padding::ZERO
            })
            .into();
        let mut ui = Simulator::with_size(iced::Settings::default(), Size::new(300.0, 300.0), root);

        // Open the menu: the pick list sits at (0, 100), ~31 px tall.
        ui.point_at(Point::new(10.0, 110.0));
        let _ = ui.simulate(iced_test::simulator::click());
        // Draw with the menu and the bubble both showing.
        let _ = ui.snapshot(&iced::Theme::Light).expect("draws");

        // The menu's single option spans roughly y ∈ [131, 162]; hovering it
        // then clicking selects it, only possible if the menu overlay exists.
        let option = Point::new(10.0, 145.0);
        ui.point_at(option);
        let _ = ui.simulate([Event::Mouse(mouse::Event::CursorMoved { position: option })]);
        let _ = ui.simulate(iced_test::simulator::click());

        assert_eq!(
            ui.into_messages().collect::<Vec<_>>(),
            vec![Message::Picked("only option")]
        );
    }
}
