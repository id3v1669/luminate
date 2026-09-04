//! Animated sizing for any child, including stock iced widgets.
//!
//! Most iced widgets take their `Length` and `Padding` when they are
//! *constructed*, which means those values are captured while the view is
//! being built. An animated value read there is a snapshot: it freezes until
//! the application next rebuilds the view, which is exactly what the motion
//! engine exists to avoid.
//!
//! [`Sized`] closes that gap. It owns the layout for its child and resolves
//! its own [`AnimLength`] values inside `layout`, so the child is re-measured
//! against fresh numbers every frame:
//!
//! ```
//! use iced::widget::text;
//! use iced::Padding;
//! use iced_animate::widget::sized;
//! use iced_animate::{curves::SMOOTH, key, motion_set, Motion};
//!
//! motion_set! {
//!     struct RowStyle -> RowStyleAnim {
//!         row_height: f32,
//!         row_pad: Padding,
//!     }
//! }
//! const OPEN: RowStyle = RowStyle { row_height: 48.0, row_pad: Padding::new(8.0) };
//! const CLOSED: RowStyle = RowStyle { row_height: 24.0, row_pad: Padding::ZERO };
//!
//! let m = Motion::new();
//! let open = true;
//! let s = m.to_set(key!(), SMOOTH, if open { OPEN } else { CLOSED });
//!
//! let _: iced::Element<'_, ()> = sized(text("row"))
//!     .height(s.row_height)
//!     .padding(s.row_pad)
//!     .into();
//! ```
//!
//! The child needs to know nothing about animation.

use iced_core::widget::{Tree, tree};
use iced_core::{Clipboard, Layout, Shell, Widget, layout, mouse, overlay, renderer};
use iced_core::{Element, Event, Length, Padding, Rectangle, Size, Vector};

use crate::{Anim, AnimLength, Tier};

/// A bouncy spring may overshoot below zero; padding cannot.
fn non_negative(padding: Padding) -> Padding {
    Padding {
        top: padding.top.max(0.0),
        right: padding.right.max(0.0),
        bottom: padding.bottom.max(0.0),
        left: padding.left.max(0.0),
    }
}

/// Wraps `content` so its size and padding can be animated.
#[must_use]
pub fn sized<'a, Message, Theme, Renderer>(
    content: impl Into<Element<'a, Message, Theme, Renderer>>,
) -> Sized<'a, Message, Theme, Renderer>
where
    Renderer: renderer::Renderer,
{
    Sized::new(content)
}

/// A container whose width, height and padding are resolved every frame.
///
/// See the [`widget`](crate::widget) module for why this is needed at all.
pub struct Sized<'a, Message, Theme, Renderer>
where
    Renderer: renderer::Renderer,
{
    width: AnimLength,
    height: AnimLength,
    padding: Anim<Padding>,
    collapse: Anim<f32>,
    content: Element<'a, Message, Theme, Renderer>,
}

impl<Message, Theme, Renderer> std::fmt::Debug for Sized<'_, Message, Theme, Renderer>
where
    Renderer: renderer::Renderer,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Sized")
            .field("width", &self.width)
            .field("height", &self.height)
            .field("padding", &self.padding)
            .field("collapse", &self.collapse)
            .finish_non_exhaustive()
    }
}

impl<'a, Message, Theme, Renderer> Sized<'a, Message, Theme, Renderer>
where
    Renderer: renderer::Renderer,
{
    /// Wraps `content`, initially sized to fit it.
    #[must_use]
    pub fn new(content: impl Into<Element<'a, Message, Theme, Renderer>>) -> Self {
        Self {
            width: AnimLength::Shrink,
            height: AnimLength::Shrink,
            padding: Anim::constant(Padding::ZERO),
            collapse: Anim::constant(1.0),
            content: content.into(),
        }
    }

    /// Sets the width, which may be an animated value.
    #[must_use]
    pub fn width(mut self, width: impl Into<AnimLength>) -> Self {
        self.width = width.into();
        self
    }

    /// Sets the height, which may be an animated value.
    #[must_use]
    pub fn height(mut self, height: impl Into<AnimLength>) -> Self {
        self.height = height.into();
        self
    }

    /// Sets the padding, which may be an animated value.
    #[must_use]
    pub fn padding(mut self, padding: impl Into<Anim<Padding>>) -> Self {
        self.padding = padding.into();
        self
    }

    /// Scales the height this widget *reports*, from its measured content
    /// height down to nothing.
    ///
    /// This is the "collapse to zero" that a plain height cannot express: the
    /// natural height of a piece of content is only known after it has been
    /// laid out, so it cannot be written down in the view as an animation
    /// target. `collapse` animates the factor instead, `1.0` is the content's
    /// own height, `0.0` is nothing, and the content keeps its natural
    /// layout, so text does not reflow on the way out.
    ///
    /// The content is anchored at the top and clipped, so the box shrinks
    /// upward and whatever follows it slides up. That is the point: unlike a
    /// fade, a collapse moves its siblings, which is why it necessarily costs
    /// a relayout per frame.
    ///
    /// Pointer input is clipped along with the paint, clipped-away content
    /// cannot be clicked or hovered, and a fully collapsed box (factor `0.0`
    /// or below) shows no overlays, a pick list's menu stays closed.
    /// Keyboard focus is not: a text input inside a fully collapsed box is
    /// still reachable by tab, so take it out of the view once its exit has
    /// finished.
    ///
    /// ```
    /// use iced::widget::text;
    /// use iced_animate::widget::sized;
    /// use iced_animate::{curves::FADE, key, Motion};
    ///
    /// let m = Motion::new();
    /// // A list row that collapses out (pair with a compositor-tier opacity
    /// // to fade it at the same time).
    /// let leaving = m.retire(key!(), FADE, 0.0_f32);
    ///
    /// let _: iced::Element<'_, ()> = sized(text("row")).collapse(leaving).into();
    /// ```
    #[must_use]
    pub fn collapse(mut self, factor: impl Into<Anim<f32>>) -> Self {
        self.collapse = factor.into();
        self
    }

    /// Flags every animated value here as one the layout depends on.
    fn mark_tiers(&self) {
        self.width.mark_layout_tier();
        self.height.mark_layout_tier();

        self.padding.mark_tier(Tier::Layout);
        self.collapse.mark_tier(Tier::Layout);
    }

    /// The cursor as the content is allowed to see it.
    ///
    /// A collapsed box is smaller than the content laid out inside it, and
    /// that content deliberately keeps its original layout nodes, that is
    /// what lets it slide up behind the clip instead of being squashed. Its
    /// *hit-boxes* keep those positions too, so without this a list row that
    /// has finished leaving still sits over whatever moved up into its place
    /// and swallows the click meant for its neighbour.
    ///
    /// `levitate` is the iced idiom for "clipped away": the position survives,
    /// so a widget already mid-press still receives its release, but every
    /// `is_over` / `position_over` test reports nothing.
    fn clipped_cursor(&self, bounds: Rectangle, cursor: mouse::Cursor) -> mouse::Cursor {
        if self.collapse.get() >= 1.0 || cursor.is_over(bounds) {
            cursor
        } else {
            cursor.levitate()
        }
    }
}

impl<Message, Theme, Renderer> Widget<Message, Theme, Renderer>
    for Sized<'_, Message, Theme, Renderer>
where
    Renderer: renderer::Renderer,
{
    fn tag(&self) -> tree::Tag {
        tree::Tag::stateless()
    }

    fn state(&self) -> tree::State {
        tree::State::None
    }

    fn children(&self) -> Vec<Tree> {
        vec![Tree::new(&self.content)]
    }

    fn diff(&self, tree: &mut Tree) {
        tree.diff_children(std::slice::from_ref(&self.content));
    }

    fn size(&self) -> Size<Length> {
        Size::new(self.width.resolve(), self.height.resolve())
    }

    fn size_hint(&self) -> Size<Length> {
        // Not the same as `size`: an animated axis must never be advertised as
        // `Fixed(0.0)`, or the parent container drops this widget outright.
        // See [`AnimLength::size_hint`].
        // A collapsing height is derived from the content, so there is no
        // number to advertise, and at factor 0 a concrete one would be the
        // void hint that gets this widget deleted.
        let height = if self.collapse.is_live() {
            Length::Shrink
        } else {
            self.height.size_hint()
        };

        Size::new(self.width.size_hint(), height)
    }

    fn layout(
        &mut self,
        tree: &mut Tree,
        renderer: &Renderer,
        limits: &layout::Limits,
    ) -> layout::Node {
        self.mark_tiers();

        let width = self.width.resolve();
        let height = self.height.resolve();
        let padding = non_negative(self.padding.get());

        // `layout::padded`, inlined: the collapsed box is then built in the
        // same pass instead of cloning the content's subtree out of a
        // finished node every frame of a collapse.
        let limits = limits.width(width).height(height);
        let content = self.content.as_widget_mut().layout(
            &mut tree.children[0],
            renderer,
            &limits.shrink(padding),
        );
        let padding = padding.fit(content.size(), limits.max());
        let size = limits
            .shrink(padding)
            .resolve(width, height, content.size())
            .expand(padding);

        // Keep the content exactly where it is and shrink only the box
        // around it, so it slides up behind the clip rather than being
        // squashed. A non-finite factor (only possible from a constant; the
        // engine sanitises its tracks) means "not collapsed".
        let factor = self.collapse.get();
        let size = if factor.is_finite() && factor < 1.0 {
            Size::new(size.width, size.height * factor.max(0.0))
        } else {
            size
        };

        layout::Node::with_children(size, vec![content.move_to((padding.left, padding.top))])
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
        let Some(content_layout) = layout.children().next() else {
            return;
        };

        let cursor = self.clipped_cursor(layout.bounds(), cursor);

        self.content.as_widget_mut().update(
            &mut tree.children[0],
            event,
            content_layout,
            cursor,
            renderer,
            clipboard,
            shell,
            viewport,
        );
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
        let Some(content_layout) = layout.children().next() else {
            return;
        };

        // Clipped-away content must not paint itself hovered either.
        let cursor = self.clipped_cursor(layout.bounds(), cursor);

        let draw = |renderer: &mut Renderer| {
            self.content.as_widget().draw(
                &tree.children[0],
                renderer,
                theme,
                style,
                content_layout,
                cursor,
                viewport,
            );
        };

        // A collapsed box is smaller than its content, so the overflow has to
        // be clipped or it would spill over whatever comes next.
        if self.collapse.get() < 1.0 {
            renderer.with_layer(layout.bounds(), draw);
        } else {
            draw(renderer);
        }
    }

    fn operate(
        &mut self,
        tree: &mut Tree,
        layout: Layout<'_>,
        renderer: &Renderer,
        operation: &mut dyn iced_core::widget::Operation,
    ) {
        let Some(content_layout) = layout.children().next() else {
            return;
        };

        self.content.as_widget_mut().operate(
            &mut tree.children[0],
            content_layout,
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
        let cursor = self.clipped_cursor(layout.bounds(), cursor);

        layout
            .children()
            .next()
            .map_or_else(mouse::Interaction::default, |content_layout| {
                self.content.as_widget().mouse_interaction(
                    &tree.children[0],
                    content_layout,
                    cursor,
                    viewport,
                    renderer,
                )
            })
    }

    fn overlay<'b>(
        &'b mut self,
        tree: &'b mut Tree,
        layout: Layout<'b>,
        renderer: &Renderer,
        viewport: &Rectangle,
        translation: Vector,
    ) -> Option<overlay::Element<'b, Message, Theme, Renderer>> {
        // Nothing of a fully collapsed box is visible; its overlays must not
        // float above whatever slid up into its place.
        if self.collapse.get() <= 0.0 {
            return None;
        }

        let content_layout = layout.children().next()?;

        self.content.as_widget_mut().overlay(
            &mut tree.children[0],
            content_layout,
            renderer,
            viewport,
            translation,
        )
    }
}

impl<'a, Message, Theme, Renderer> From<Sized<'a, Message, Theme, Renderer>>
    for Element<'a, Message, Theme, Renderer>
where
    Message: 'a,
    Theme: 'a,
    Renderer: renderer::Renderer + 'a,
{
    fn from(sized: Sized<'a, Message, Theme, Renderer>) -> Self {
        Element::new(sized)
    }
}

// `iced_core` implements `Renderer for ()` only in debug builds.
#[cfg(all(test, debug_assertions))]
mod tests {
    use iced_core::widget::Tree;
    use iced_core::{Length, Padding, Point, Rectangle, Size, Widget, layout};

    use super::{Sized, sized};
    use crate::widget::shape;
    use crate::{Anim, Motion, curves::QUICK, key};

    type Boxed<'a> = Sized<'a, (), (), ()>;

    fn layout_of(widget: &mut Boxed<'_>) -> layout::Node {
        let mut tree = Tree::new(&*widget as &dyn Widget<(), (), ()>);
        Widget::layout(
            widget,
            &mut tree,
            &(),
            &layout::Limits::new(Size::ZERO, Size::new(400.0, 400.0)),
        )
    }

    fn content() -> Boxed<'static> {
        sized(shape().width(100.0).height(40.0)).padding(Padding::new(10.0))
    }

    #[test]
    fn a_collapse_factor_shrinks_the_box_and_leaves_the_content_in_place() {
        let mut full = content();
        let node = layout_of(&mut full);
        assert_eq!(node.size(), Size::new(120.0, 60.0));

        let mut half = content().collapse(0.5);
        let node = layout_of(&mut half);
        assert_eq!(
            node.size(),
            Size::new(120.0, 30.0),
            "half the padded height"
        );
        assert_eq!(
            node.children()[0].bounds(),
            Rectangle::new(Point::new(10.0, 10.0), Size::new(100.0, 40.0)),
            "the content keeps its natural layout behind the clip"
        );
    }

    #[test]
    fn a_non_finite_collapse_factor_is_ignored() {
        let mut nan = content().collapse(f32::NAN);
        assert_eq!(layout_of(&mut nan).size(), Size::new(120.0, 60.0));
        let mut negative = content().collapse(-1.0);
        assert_eq!(layout_of(&mut negative).size(), Size::new(120.0, 0.0));
    }

    #[test]
    fn negative_padding_is_clamped_to_zero() {
        let mut widget = sized(shape().width(100.0).height(40.0)).padding(Padding::new(-5.0));
        let node = layout_of(&mut widget);
        assert_eq!(node.size(), Size::new(100.0, 40.0));
        assert_eq!(node.children()[0].bounds().x, 0.0);
    }

    #[test]
    fn a_live_collapse_hints_shrink_but_a_constant_height_hints_itself() {
        let m = Motion::new();
        let key = key!();
        let _ = m.to(key, QUICK, 1.0_f32);
        let live = m.to(key, QUICK, 0.0_f32);

        let collapsing: Boxed<'_> = sized(shape()).height(40.0).collapse(live);
        assert_eq!(
            Widget::<(), (), ()>::size_hint(&collapsing).height,
            Length::Shrink
        );

        let steady: Boxed<'_> = sized(shape()).height(40.0).collapse(Anim::constant(0.5));
        assert_eq!(
            Widget::<(), (), ()>::size_hint(&steady).height,
            Length::Fixed(40.0)
        );
    }
}

#[cfg(test)]
/// Tests that drive the widget through `iced_test`.
mod simulator_tests {
    use std::time::Duration;

    use crate::{Curve, Motion, SpringParams, key};

    /// A fast spring for tests; not the shipped `curves::SMOOTH`.
    const FAST: Curve = Curve::spring(SpringParams::new(0.0, Duration::from_millis(300)));

    /// A row collapsed to nothing keeps its children's layout nodes exactly where
    /// they were, that is what lets the content slide up behind the clip instead
    /// of being squashed. It must not keep their hit-boxes too: a list row that
    /// has finished leaving sits on top of whatever moved up into its place, and
    /// would swallow the click meant for its neighbour.
    #[test]
    fn a_collapsed_row_does_not_intercept_clicks() {
        use iced::widget::{button, column};

        #[derive(Debug, Clone, PartialEq)]
        enum Message {
            Ghost,
            Live,
        }

        let m = Motion::new();

        // The gone row: collapsed to zero, still built because nothing has
        // rebuilt the view since its exit finished.
        let gone: iced::Element<'_, Message> =
            crate::widget::sized(button("gone").on_press(Message::Ghost))
                .collapse(m.to(key!(), FAST, 0.0_f32))
                .into();

        let live: iced::Element<'_, Message> =
            crate::widget::sized(button("live").on_press(Message::Live)).into();

        let mut ui: iced_test::Simulator<'_, Message> = iced_test::Simulator::with_size(
            iced::Settings::default(),
            iced::Size::new(400.0, 200.0),
            column![gone, live].width(iced::Length::Fill),
        );

        let _ = ui.click("live").expect("the live button is on screen");

        assert_eq!(
            ui.into_messages().collect::<Vec<_>>(),
            vec![Message::Live],
            "the click landed on the collapsed row above instead"
        );
    }
}
