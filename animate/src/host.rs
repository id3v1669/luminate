//! The single point where animation time advances.
//!
//! [`Host`] wraps the application root. It ticks the engine on
//! `RedrawRequested` *before* forwarding the event, so every widget below
//! reads this frame's values, and it is the only place that decides whether
//! the frame needs a relayout or merely a redraw. Keeping one clock here means
//! there is exactly one place a frame delta is derived.

use iced_core::widget::{Tree, tree};
use iced_core::{Clipboard, Layout, Shell, Widget, layout, mouse, overlay, renderer};
use iced_core::{Element, Event, Length, Rectangle, Size, Vector, window};

use crate::Motion;
use crate::engine::HostId;

/// Wraps the application root and advances the [`Motion`] engine each frame.
///
/// Place this at the very top of your view. Everything that animates must live
/// inside it, because a widget only sees fresh values if the tick has already
/// run when its own `update` is called.
///
/// Use exactly one per view, and one [`Motion`] per window. A second host in
/// the same view is ignored for timing (the engine advances once per
/// timestamp) but still counts as a view build, halving the garbage
/// collector's patience; the engine logs a warning once, in every build
/// profile. Two windows sharing one engine trip the same warning, because
/// their frames are distinct builds ticking distinct hosts.
pub struct Host<'a, Message, Theme, Renderer>
where
    Renderer: renderer::Renderer,
{
    motion: Motion,
    id: HostId,
    content: Element<'a, Message, Theme, Renderer>,
}

impl<Message, Theme, Renderer> std::fmt::Debug for Host<'_, Message, Theme, Renderer>
where
    Renderer: renderer::Renderer,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Host")
            .field("motion", &self.motion)
            .field("id", &self.id)
            .finish_non_exhaustive()
    }
}

impl<'a, Message, Theme, Renderer> Host<'a, Message, Theme, Renderer>
where
    Renderer: renderer::Renderer,
{
    /// Wraps `content` so that `motion` is ticked once per frame.
    ///
    /// Constructing the host is also what closes a `view()` build for the
    /// engine's garbage collector: `content` is fully built by the time this
    /// runs, so every track the build referenced has been touched.
    #[must_use]
    pub fn new(motion: &Motion, content: impl Into<Element<'a, Message, Theme, Renderer>>) -> Self {
        let content = content.into();
        motion.end_build();

        Self {
            motion: motion.clone(),
            id: HostId::next(),
            content,
        }
    }
}

impl Motion {
    /// Wraps `content` in a [`Host`] bound to this engine.
    ///
    /// ```no_run
    /// # use iced::widget::column;
    /// # use iced_animate::Motion;
    /// # fn view(motion: &Motion) -> iced::Element<'_, ()> {
    /// motion.host(column![]).into()
    /// # }
    /// ```
    #[must_use]
    pub fn host<'a, Message, Theme, Renderer>(
        &self,
        content: impl Into<Element<'a, Message, Theme, Renderer>>,
    ) -> Host<'a, Message, Theme, Renderer>
    where
        Renderer: renderer::Renderer,
    {
        Host::new(self, content)
    }
}

/// Wraps `content` so that `motion` is ticked once per frame.
///
/// Free-function form of [`Host::new`] and [`Motion::host`], for use inside a
/// `view()` chain.
#[must_use]
pub fn host<'a, Message, Theme, Renderer>(
    motion: &Motion,
    content: impl Into<Element<'a, Message, Theme, Renderer>>,
) -> Host<'a, Message, Theme, Renderer>
where
    Renderer: renderer::Renderer,
{
    Host::new(motion, content)
}

impl<Message, Theme, Renderer> Widget<Message, Theme, Renderer>
    for Host<'_, Message, Theme, Renderer>
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
        self.content.as_widget().size()
    }

    fn size_hint(&self) -> Size<Length> {
        self.content.as_widget().size_hint()
    }

    fn layout(
        &mut self,
        tree: &mut Tree,
        renderer: &Renderer,
        limits: &layout::Limits,
    ) -> layout::Node {
        self.content
            .as_widget_mut()
            .layout(&mut tree.children[0], renderer, limits)
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
        if let Event::Window(window::Event::RedrawRequested(now)) = event {
            let status = self.motion.tick_from(*now, Some(self.id));

            // A moving `Layout`-tier track invalidates the layout; a moving
            // transform or opacity does not. Keeping these separate is what
            // stops a pure fade from dragging a full relayout behind it.
            if status.layout_invalid {
                shell.invalidate_layout();
            }

            if status.animating {
                shell.request_redraw();
            }

            self.motion.gc_if_built();
        }

        self.content.as_widget_mut().update(
            &mut tree.children[0],
            event,
            layout,
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
        self.content.as_widget().draw(
            &tree.children[0],
            renderer,
            theme,
            style,
            layout,
            cursor,
            viewport,
        );
    }

    fn operate(
        &mut self,
        tree: &mut Tree,
        layout: Layout<'_>,
        renderer: &Renderer,
        operation: &mut dyn iced_core::widget::Operation,
    ) {
        self.content
            .as_widget_mut()
            .operate(&mut tree.children[0], layout, renderer, operation);
    }

    fn mouse_interaction(
        &self,
        tree: &Tree,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        viewport: &Rectangle,
        renderer: &Renderer,
    ) -> mouse::Interaction {
        self.content.as_widget().mouse_interaction(
            &tree.children[0],
            layout,
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
        self.content.as_widget_mut().overlay(
            &mut tree.children[0],
            layout,
            renderer,
            viewport,
            translation,
        )
    }
}

impl<'a, Message, Theme, Renderer> From<Host<'a, Message, Theme, Renderer>>
    for Element<'a, Message, Theme, Renderer>
where
    Message: 'a,
    Theme: 'a,
    Renderer: renderer::Renderer + 'a,
{
    fn from(host: Host<'a, Message, Theme, Renderer>) -> Self {
        Element::new(host)
    }
}

// `iced_core` implements `Renderer for ()` only in debug builds.
#[cfg(all(test, debug_assertions))]
mod tests {
    use std::time::Duration;

    use iced_core::time::Instant;
    use iced_core::widget::Tree;
    use iced_core::window::RedrawRequest;
    use iced_core::{
        Color, Event, Rectangle, Shell, Size, Widget, clipboard, layout, mouse, window,
    };

    use super::Host;
    use crate::engine::GC_IDLE_BUILDS;
    use crate::widget::shape;
    use crate::{Motion, curves::QUICK, key};

    type Root<'a> = Host<'a, (), (), ()>;

    const VIEWPORT: Rectangle = Rectangle {
        x: 0.0,
        y: 0.0,
        width: 400.0,
        height: 200.0,
    };

    /// Sends one `RedrawRequested` through the host and reports what it asked
    /// the shell for.
    fn redraw(host: &mut Root<'_>, tree: &mut Tree, at: Instant) -> (RedrawRequest, bool) {
        let node = Widget::layout(
            host,
            tree,
            &(),
            &layout::Limits::new(Size::ZERO, VIEWPORT.size()),
        );
        let mut messages = Vec::new();
        let mut shell = Shell::new(&mut messages);

        Widget::update(
            host,
            tree,
            &Event::Window(window::Event::RedrawRequested(at)),
            layout::Layout::new(&node),
            mouse::Cursor::Unavailable,
            &(),
            &mut clipboard::Null,
            &mut shell,
            &VIEWPORT,
        );

        (shell.redraw_request(), shell.is_layout_invalid())
    }

    #[test]
    fn the_host_requests_frames_while_moving_and_relayouts_only_for_layout_tracks() {
        let motion = Motion::new();
        let fill_key = key!();
        let size_key = key!();
        let _ = motion.to(fill_key, QUICK, Color::BLACK);
        let _ = motion.to(size_key, QUICK, 40.0_f32);
        let fill = motion.to(fill_key, QUICK, Color::WHITE);
        let side = motion.to(size_key, QUICK, 120.0_f32);

        let mut host: Root<'_> = Host::new(&motion, shape().width(&side).height(40.0).fill(fill));
        let mut tree = Tree::new(&host as &dyn Widget<(), (), ()>);
        let start = Instant::now();

        let (request, relayout) = redraw(&mut host, &mut tree, start);
        assert_eq!(
            request,
            RedrawRequest::NextFrame,
            "the clock starts; more frames wanted"
        );
        assert!(!relayout, "nothing moved on the first frame");

        let (request, relayout) = redraw(&mut host, &mut tree, start + Duration::from_millis(16));
        assert_eq!(request, RedrawRequest::NextFrame);
        assert!(relayout, "the width is a layout-tier track");

        let mut settled_at = None;
        for frame in 2..400_u64 {
            let at = start + Duration::from_millis(16 * frame);
            let (request, _) = redraw(&mut host, &mut tree, at);
            if request == RedrawRequest::Wait {
                settled_at = Some(frame);
                break;
            }
        }
        let frame = settled_at.expect("a 220 ms spring settles within 6 s");
        assert!(frame > 5, "not instantly: {frame}");
        assert_eq!(side.get(), 120.0);

        // At rest: no frame, no relayout.
        let (request, relayout) = redraw(
            &mut host,
            &mut tree,
            start + Duration::from_millis(16 * (frame + 1)),
        );
        assert_eq!(request, RedrawRequest::Wait);
        assert!(!relayout);
    }

    #[test]
    fn a_paint_only_animation_never_invalidates_the_layout() {
        let motion = Motion::new();
        let key = key!();
        let _ = motion.to(key, QUICK, Color::BLACK);
        let fill = motion.to(key, QUICK, Color::WHITE);

        let mut host: Root<'_> = Host::new(&motion, shape().width(40.0).height(40.0).fill(fill));
        let mut tree = Tree::new(&host as &dyn Widget<(), (), ()>);
        let start = Instant::now();

        for frame in 0..30_u64 {
            let (_, relayout) = redraw(
                &mut host,
                &mut tree,
                start + Duration::from_millis(16 * frame),
            );
            assert!(!relayout, "frame {frame} asked for a relayout");
        }
    }

    #[test]
    fn each_host_build_ages_unreferenced_tracks_out() {
        let motion = Motion::new();
        let _ = motion.to(key!(), QUICK, 1.0_f32); // settled, handle dropped
        assert_eq!(motion.track_count(), 1);

        let start = Instant::now();
        for build in 0..=GC_IDLE_BUILDS {
            let mut host: Root<'_> = Host::new(&motion, shape().width(10.0).height(10.0));
            let mut tree = Tree::new(&host as &dyn Widget<(), (), ()>);
            let _ = redraw(
                &mut host,
                &mut tree,
                start + Duration::from_millis(16 * build),
            );
        }

        assert_eq!(
            motion.track_count(),
            0,
            "collected once per build, not per frame"
        );
    }
}
