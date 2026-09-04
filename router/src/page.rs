//! The page contract.

use std::any::Any;

use iced_core::Element;
use iced_runtime::futures::Subscription;

use crate::action::Action;
use crate::registry::Registry;

/// What happens to a page's instance when the router leaves it.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
#[non_exhaustive]
pub enum Lifecycle {
    /// The instance is built on first visit and dropped when left;
    /// [`Page::into_snapshot`] runs on the way out and the result is handed to
    /// [`Page::restore`] on the next visit. The default.
    #[default]
    Drop,
    /// The instance is built on first visit and kept alive when left; it is
    /// told via [`Page::on_suspend`] and [`Page::on_resume`].
    Suspend,
    /// The instance is built when the page is [added](crate::Router::add)
    /// and lives as long as the router; otherwise like [`Suspend`](Self::Suspend).
    /// Use it for a page whose
    /// [`background_subscription`](Page::background_subscription) must run
    /// for the whole application.
    Resident,
}

/// A page the router can show.
///
/// # Lifecycle, in order
///
/// - First visit: [`new`] → ([`restore`] if a snapshot exists) → [`on_enter`]
///   → ([`on_navigate`] if the navigation carried options).
/// - Leaving: [`Lifecycle::Drop`] → [`into_snapshot`], instance dropped;
///   [`Lifecycle::Suspend`]/[`Resident`] → [`on_suspend`].
/// - Returning (navigate, back, forward): `Drop` → `new`, `restore`,
///   `on_enter`; `Suspend`/`Resident` → [`on_resume`], `on_enter`; then
///   `on_navigate` if options were carried.
/// - Navigating to the page that is already current fires only
///   `on_navigate` (when options were carried), the page did not *become*
///   current.
///
/// # Messages and tasks
///
/// A [`Task`](iced_runtime::Task) returned from [`update`] is delivered to
/// the instance that returned it. If that instance has been dropped (a
/// `Drop` page that was left) the result is discarded with a warning, even
/// if the page has been re-created since. For long work, make the page
/// `Suspend`, or let the *target* page start it: record the options in
/// `on_navigate` and act on them from [`update`] (via a message) or from
/// [`subscription`](Self::subscription).
///
/// [`new`]: Self::new
/// [`restore`]: Self::restore
/// [`on_enter`]: Self::on_enter
/// [`on_navigate`]: Self::on_navigate
/// [`into_snapshot`]: Self::into_snapshot
/// [`on_suspend`]: Self::on_suspend
/// [`on_resume`]: Self::on_resume
/// [`update`]: Self::update
/// [`Resident`]: Lifecycle::Resident
pub trait Page: 'static {
    /// Messages this page handles.
    ///
    /// `Clone` because iced clones messages (`on_press`, `Subscription`);
    /// `Send + Sync` because the router wraps it in an `Arc` inside a
    /// [`RouteMessage`](crate::RouteMessage), which must be `Send` for
    /// [`Task`](iced_runtime::Task).
    type Message: Clone + Send + Sync + 'static;
    /// Payload accepted by [`on_navigate`](Self::on_navigate). Use `()` for
    /// none. Same bounds as `Message` because options travel inside a
    /// [`RouteMessage`](crate::RouteMessage).
    type NavigationOptions: Clone + Send + Sync + 'static;
    /// What the router hands every page at construction (a UI kit, a
    /// database handle, …).
    type Context: 'static;
    /// The theme the page's view is drawn with.
    type Theme: 'static;
    /// The renderer the page's view is drawn with.
    type Renderer: iced_core::Renderer + 'static;

    /// See [`Lifecycle`].
    const LIFECYCLE: Lifecycle = Lifecycle::Drop;

    /// Builds the page.
    fn new(context: &Self::Context, registry: &Registry) -> Self;

    /// Handles one message.
    fn update(&mut self, message: Self::Message) -> Action<Self::Message>;

    /// Builds the page's view.
    fn view(&self) -> Element<'_, Self::Message, Self::Theme, Self::Renderer>;

    /// Subscription active while this page is current.
    fn subscription(&self) -> Subscription<Self::Message> {
        Subscription::none()
    }

    /// Subscription active while the page *instance exists*, for the whole
    /// application with [`Lifecycle::Resident`].
    fn background_subscription(&self) -> Subscription<Self::Message> {
        Subscription::none()
    }

    /// Called every time the page becomes current: navigate, back, forward.
    fn on_enter(&mut self) {}

    /// Called with the options a navigation carried, after
    /// [`on_enter`](Self::on_enter). Not called for option-less navigations
    /// or history moves.
    fn on_navigate(&mut self, _options: Self::NavigationOptions) {}

    /// Called when a [`Lifecycle::Suspend`]/[`Resident`](Lifecycle::Resident)
    /// page is left.
    fn on_suspend(&mut self) {}

    /// Called when a [`Lifecycle::Suspend`]/[`Resident`](Lifecycle::Resident)
    /// page is returned to, before [`on_enter`](Self::on_enter).
    fn on_resume(&mut self) {}

    /// State to keep when a [`Lifecycle::Drop`] page is left. The instance
    /// is consumed, so move fields out instead of cloning them.
    fn into_snapshot(self) -> Option<Box<dyn Any>>
    where
        Self: Sized,
    {
        None
    }

    /// Receives what [`into_snapshot`](Self::into_snapshot) returned, on the
    /// next visit, right after [`new`](Self::new).
    fn restore(&mut self, _snapshot: Box<dyn Any>) {}
}
