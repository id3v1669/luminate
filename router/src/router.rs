//! The router: a list of pages, one of which is current.

use std::any::{Any, TypeId};
use std::collections::HashMap;

use iced_core::event::Status;
use iced_core::{Element, Event, mouse, window};
use iced_runtime::Task;
use iced_runtime::futures::{Subscription, event};

use crate::erased::{Erased, ErasedPage, programming_error};
use crate::error::NavigationError;
use crate::history::History;
use crate::message::{Navigation, PageMessage, Payload, RouteMessage};
use crate::page::{Lifecycle, Page};
use crate::registry::Registry;

/// Entries a router's history keeps unless [`Router::history_len`] says
/// otherwise.
const DEFAULT_HISTORY_LEN: usize = 50;

type Instance<Theme, Renderer> = Box<dyn ErasedPage<Theme, Renderer>>;
type Factory<Ctx, Theme, Renderer> =
    Box<dyn Fn(&Ctx, &Registry, usize, u64) -> Instance<Theme, Renderer>>;

struct Entry<Ctx, Theme, Renderer> {
    factory: Factory<Ctx, Theme, Renderer>,
    instance: Option<Instance<Theme, Renderer>>,
    snapshot: Option<Box<dyn Any>>,
    lifecycle: Lifecycle,
    /// Bumped every time an instance is dropped, so the next instance (and
    /// the messages it emits) can be told apart from the old one.
    generation: u64,
    suspended: bool,
    name: String,
}

/// What [`Router::pages`] yields for each page.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub struct PageInfo<'a> {
    /// Position in the order pages were added, what
    /// [`Navigation::index`] takes.
    pub index: usize,
    /// The display name given to [`Router::add`]. Opaque to the router.
    pub name: &'a str,
    /// Whether this is the current page.
    pub is_current: bool,
    /// The page type's [`Page::LIFECYCLE`].
    pub lifecycle: Lifecycle,
}

/// A list of pages, one of which is current, with back/forward history.
///
/// `Ctx` is handed to every page's [`Page::new`]; `Theme` and `Renderer` are
/// what the pages draw with. There is no page until the first navigation:
/// [`current`](Self::current) is `None` and [`view`](Self::view) yields
/// `None`.
///
/// # History
///
/// - Bounded: the oldest entry is forgotten past the bound (default 50,
///   [`history_len`](Self::history_len)).
/// - The same page is never recorded twice in a row; navigating to the
///   current page adds nothing.
/// - Any navigation ([`navigate`](Self::navigate), [`navigate_with`](Self::navigate_with),
///   [`navigate_index`](Self::navigate_index), or a [`Navigation`] through
///   [`update`](Self::update)) discards the forward branch, also when the
///   target is the page under the cursor after a [`back`](Self::back).
/// - [`replace`](Self::replace) swaps the current entry instead of adding
///   one.
///
/// # Messages
///
/// [`RouteMessage::Page`] carries the index and *generation* of the instance
/// that produced it. A message whose instance is gone is dropped with a
/// warning; see [`Page`] for the lifecycle. [`RouteMessage::Navigate`]
/// carrying an unknown page or index is a programming error: logged and,
/// in debug builds, a panic.
///
/// ```
/// use iced::widget::{button, text};
/// use iced_page_router::{Action, Navigation, Page, Registry, RouteMessage, Router};
///
/// struct Home;
/// impl Page for Home {
///     type Message = ();
///     type NavigationOptions = ();
///     type Context = ();
///     type Theme = iced::Theme;
///     type Renderer = iced::Renderer;
///     fn new(_: &(), _: &Registry) -> Self { Home }
///     fn update(&mut self, (): ()) -> Action<()> { Action::none() }
///     fn view(&self) -> iced::Element<'_, ()> { text("home").into() }
/// }
///
/// let mut router: Router<(), iced::Theme, iced::Renderer> = Router::new(Registry::new(), ());
/// router.add::<Home>("Home");
/// assert!(router.view().is_none());
/// assert_eq!(router.navigate::<Home>(), Ok(0));
/// assert!(router.view().is_some());
/// let _task = router.update(RouteMessage::Navigate(Navigation::back()));
/// assert!(!router.can_go_back());
/// ```
pub struct Router<Ctx, Theme, Renderer> {
    pages: Vec<Entry<Ctx, Theme, Renderer>>,
    index: HashMap<TypeId, usize>,
    current: Option<usize>,
    history: History,
    mouse_navigation: bool,
    context: Ctx,
    registry: Registry,
}

impl<Ctx, Theme, Renderer> Router<Ctx, Theme, Renderer>
where
    Ctx: 'static,
    Theme: 'static,
    Renderer: iced_core::Renderer + 'static,
{
    /// A router with no pages. Add pages with [`add`](Self::add), then
    /// navigate to the first one.
    #[must_use]
    pub fn new(registry: Registry, context: Ctx) -> Self {
        Self {
            pages: Vec::new(),
            index: HashMap::new(),
            current: None,
            history: History::new(DEFAULT_HISTORY_LEN),
            mouse_navigation: true,
            context,
            registry,
        }
    }

    /// Whether the mouse back/forward buttons move through the history
    /// (default `true`).
    ///
    /// Presses are ignored when a widget captured the event. Every window
    /// of the application is listened to. Turn it off for nested routers,
    /// or one physical press moves every router at once:
    ///
    /// ```
    /// use iced_page_router::{Registry, Router};
    ///
    /// let registry = Registry::new();
    /// let outer: Router<(), iced::Theme, iced::Renderer> = Router::new(registry.clone(), ());
    /// // Inside a page of `outer`:
    /// let inner: Router<(), iced::Theme, iced::Renderer> =
    ///     Router::new(registry.clone(), ()).mouse_navigation(false);
    /// # let _ = (outer, inner);
    /// ```
    #[must_use]
    pub fn mouse_navigation(mut self, enabled: bool) -> Self {
        self.mouse_navigation = enabled;
        self
    }

    /// How many history entries to keep (default 50).
    ///
    /// Must be called before the first navigation: it replaces the history
    /// with an empty one, so a router that already shows a page would lose
    /// its entries (including the current one) and `back` would do nothing
    /// until the next navigation.
    ///
    /// # Panics
    ///
    /// Debug builds only: panics if `len == 0`. Release builds log an error
    /// and keep one entry (the current page).
    #[must_use]
    pub fn history_len(mut self, len: usize) -> Self {
        self.history = History::new(len);
        self
    }

    /// The registry shared with every page.
    #[must_use]
    pub fn registry(&self) -> &Registry {
        &self.registry
    }

    /// The context handed to every page.
    #[must_use]
    pub fn context(&self) -> &Ctx {
        &self.context
    }

    /// Mutable access to the context. Pages built earlier keep whatever
    /// they copied out of it in [`Page::new`].
    #[must_use]
    pub fn context_mut(&mut self) -> &mut Ctx {
        &mut self.context
    }

    /// Appends page `P` under the display `name`. A
    /// [`Lifecycle::Resident`] page is built right away.
    ///
    /// # Panics
    ///
    /// Debug builds only: panics if `P` was already added, since a page type
    /// identifies a page. Release builds log an error and ignore the second
    /// `add` (the existing entry, including its name, stays).
    pub fn add<P>(&mut self, name: impl Into<String>) -> &mut Self
    where
        P: Page<Context = Ctx, Theme = Theme, Renderer = Renderer>,
    {
        if self.index.contains_key(&TypeId::of::<P>()) {
            debug_assert!(false, "page {} added twice", std::any::type_name::<P>());
            log::error!(
                "page {} added twice; the second add is ignored",
                std::any::type_name::<P>()
            );
            return self;
        }

        let id = self.pages.len();

        let factory: Factory<Ctx, Theme, Renderer> =
            Box::new(|context, registry, id, generation| {
                Box::new(Erased::new(P::new(context, registry), id, generation))
            });

        let instance = (P::LIFECYCLE == Lifecycle::Resident)
            .then(|| factory(&self.context, &self.registry, id, 0));

        self.pages.push(Entry {
            factory,
            instance,
            snapshot: None,
            lifecycle: P::LIFECYCLE,
            generation: 0,
            suspended: false,
            name: name.into(),
        });

        let _ = self.index.insert(TypeId::of::<P>(), id);

        self
    }

    /// Index of the current page, `None` before the first navigation.
    #[must_use]
    pub fn current(&self) -> Option<usize> {
        self.current
    }

    /// Every page, in the order they were added.
    pub fn pages(&self) -> impl Iterator<Item = PageInfo<'_>> {
        self.pages
            .iter()
            .enumerate()
            .map(|(index, entry)| PageInfo {
                index,
                name: entry.name.as_str(),
                is_current: self.current == Some(index),
                lifecycle: entry.lifecycle,
            })
    }

    /// Index of page `P`, if added.
    #[must_use]
    pub fn index_of<P: Page>(&self) -> Option<usize> {
        self.index.get(&TypeId::of::<P>()).copied()
    }

    /// The live instance of page `P`, if it has one right now.
    #[must_use]
    pub fn page<P: Page>(&self) -> Option<&P> {
        let id = self.index_of::<P>()?;
        self.pages[id]
            .instance
            .as_ref()?
            .as_any()
            .downcast_ref::<Erased<P>>()
            .map(|erased| &erased.page)
    }

    /// Mutable access to the live instance of page `P`, if it has one.
    #[must_use]
    pub fn page_mut<P: Page>(&mut self) -> Option<&mut P> {
        let id = self.index_of::<P>()?;
        self.pages[id]
            .instance
            .as_mut()?
            .as_any_mut()
            .downcast_mut::<Erased<P>>()
            .map(|erased| &mut erased.page)
    }

    /// Whether [`back`](Self::back) would move.
    #[must_use]
    pub fn can_go_back(&self) -> bool {
        self.history.can_go_back()
    }

    /// Whether [`forward`](Self::forward) would move.
    #[must_use]
    pub fn can_go_forward(&self) -> bool {
        self.history.can_go_forward()
    }

    /// Navigates to page `P`, records it in the history and returns its
    /// index.
    ///
    /// # Errors
    ///
    /// [`NavigationError::UnknownPage`] if `P` was never added.
    pub fn navigate<P: Page>(&mut self) -> Result<usize, NavigationError> {
        let id = self.resolve::<P>()?;
        self.visit(id, None);
        Ok(id)
    }

    /// Like [`navigate`](Self::navigate), with options for the page's
    /// [`Page::on_navigate`]. Delivers the options even if `P` is already
    /// current.
    ///
    /// # Errors
    ///
    /// [`NavigationError::UnknownPage`] if `P` was never added.
    pub fn navigate_with<P: Page>(
        &mut self,
        options: P::NavigationOptions,
    ) -> Result<usize, NavigationError> {
        let id = self.resolve::<P>()?;
        self.visit(id, Some(Payload::new(options)));
        Ok(id)
    }

    /// Navigates to the page at `index` and records it in the history.
    ///
    /// # Errors
    ///
    /// [`NavigationError::IndexOutOfRange`] if there is no such page.
    pub fn navigate_index(&mut self, index: usize) -> Result<usize, NavigationError> {
        if index >= self.pages.len() {
            return Err(NavigationError::IndexOutOfRange {
                index,
                len: self.pages.len(),
            });
        }

        self.visit(index, None);
        Ok(index)
    }

    /// Navigates to page `P`, replacing the current history entry instead
    /// of adding one (login → home, redirects).
    ///
    /// # Errors
    ///
    /// [`NavigationError::UnknownPage`] if `P` was never added.
    pub fn replace<P: Page>(&mut self) -> Result<usize, NavigationError> {
        let id = self.resolve::<P>()?;
        self.activate(id);
        self.history.replace(id);
        Ok(id)
    }

    /// Moves one step back in the history and returns the new current page.
    #[must_use]
    pub fn back(&mut self) -> Option<usize> {
        let id = self.history.back()?;
        self.activate(id);
        Some(id)
    }

    /// Moves one step forward in the history and returns the new current
    /// page.
    #[must_use]
    pub fn forward(&mut self) -> Option<usize> {
        let id = self.history.forward()?;
        self.activate(id);
        Some(id)
    }

    /// Addresses `message` to the live instance of page `P`.
    ///
    /// `None` if `P` was never added or has no instance right now (the
    /// message would be dropped as stale anyway).
    ///
    /// ```
    /// # use iced::widget::text;
    /// # use iced_page_router::{Action, Page, Registry, Router};
    /// # #[derive(Clone)] enum Msg { Refresh }
    /// # struct Home;
    /// # impl Page for Home {
    /// #     type Message = Msg; type NavigationOptions = (); type Context = ();
    /// #     type Theme = iced::Theme; type Renderer = iced::Renderer;
    /// #     fn new(_: &(), _: &Registry) -> Self { Home }
    /// #     fn update(&mut self, _: Msg) -> Action<Msg> { Action::none() }
    /// #     fn view(&self) -> iced::Element<'_, Msg> { text("").into() }
    /// # }
    /// let mut router: Router<(), iced::Theme, iced::Renderer> = Router::new(Registry::new(), ());
    /// router.add::<Home>("Home");
    /// assert!(router.message::<Home>(Msg::Refresh).is_none(), "not built yet");
    /// router.navigate::<Home>().unwrap();
    /// let message = router.message::<Home>(Msg::Refresh).unwrap();
    /// let _task = router.update(message);
    /// ```
    #[must_use]
    pub fn message<P: Page>(&self, message: P::Message) -> Option<RouteMessage> {
        let id = self.index_of::<P>()?;
        let instance = self.pages[id].instance.as_ref()?;
        Some(RouteMessage::Page(PageMessage::new(
            id,
            instance.generation(),
            message,
        )))
    }

    /// The subscriptions of the current page and of every live instance's
    /// background subscription, plus mouse back/forward if enabled.
    pub fn subscription(&self) -> Subscription<RouteMessage> {
        let background = self
            .pages
            .iter()
            .filter_map(|entry| entry.instance.as_ref())
            .map(|instance| instance.background_subscription());

        let current = self
            .current
            .and_then(|id| self.pages[id].instance.as_ref())
            .map(|instance| instance.subscription());

        let mouse = self
            .mouse_navigation
            .then(|| event::listen_with(mouse_navigation));

        Subscription::batch(background.chain(current).chain(mouse))
    }

    /// Delivers a message.
    ///
    /// A page message is delivered only to the instance whose generation it
    /// carries; otherwise it is dropped with a `warn`. A navigation returned
    /// by *any* page, the current one, a suspended one reacting to its
    /// background subscription, or a late task result, is honoured.
    #[must_use = "the task carries the page's work; hand it to the runtime"]
    pub fn update(&mut self, message: RouteMessage) -> Task<RouteMessage> {
        match message {
            RouteMessage::Page(message) => {
                let live = self
                    .pages
                    .get_mut(message.page_id())
                    .and_then(|entry| entry.instance.as_mut())
                    .filter(|instance| instance.generation() == message.generation());

                let Some(instance) = live else {
                    log::warn!(
                        "dropped a message for page {} generation {}: that instance is gone",
                        message.page_id(),
                        message.generation()
                    );
                    return Task::none();
                };

                let (task, navigation) = instance.update(&message);

                if let Some(navigation) = navigation {
                    self.apply(navigation);
                }

                task
            }
            RouteMessage::Navigate(navigation) => {
                self.apply(navigation);
                Task::none()
            }
        }
    }

    /// The current page's view; `None` before the first navigation.
    #[must_use]
    pub fn view(&self) -> Option<Element<'_, RouteMessage, Theme, Renderer>> {
        let id = self.current?;
        self.pages[id]
            .instance
            .as_ref()
            .map(|instance| instance.view())
    }

    fn resolve<P: Page>(&self) -> Result<usize, NavigationError> {
        self.index_of::<P>().ok_or(NavigationError::UnknownPage {
            type_name: std::any::type_name::<P>(),
        })
    }

    /// Applies a navigation that arrived as a message.
    fn apply(&mut self, navigation: Navigation) {
        match navigation {
            Navigation::To {
                target,
                type_name,
                options,
            } => match self.index.get(&target).copied() {
                Some(id) => self.visit(id, options),
                None => programming_error(format_args!(
                    "navigation to {type_name}, which is not in this router; ignored"
                )),
            },
            Navigation::Index(index) => {
                if let Err(error) = self.navigate_index(index) {
                    programming_error(format_args!("{error}; ignored"));
                }
            }
            Navigation::Back => {
                let _ = self.back();
            }
            Navigation::Forward => {
                let _ = self.forward();
            }
            Navigation::Replace { target, type_name } => match self.index.get(&target).copied() {
                Some(id) => {
                    self.activate(id);
                    self.history.replace(id);
                }
                None => programming_error(format_args!(
                    "replace with {type_name}, which is not in this router; ignored"
                )),
            },
        }
    }

    /// Makes `to` current, delivers `options`, and records the visit.
    fn visit(&mut self, to: usize, options: Option<Payload>) {
        self.activate(to);

        if let Some(options) = options
            && let Some(instance) = self.pages[to].instance.as_mut()
        {
            instance.on_navigate(&options);
        }

        self.history.push(to);
    }

    /// Makes `to` current without touching the history. No-op if it already
    /// is.
    fn activate(&mut self, to: usize) {
        if self.current == Some(to) {
            return;
        }

        if let Some(from) = self.current {
            let entry = &mut self.pages[from];

            match entry.lifecycle {
                Lifecycle::Suspend | Lifecycle::Resident => {
                    if let Some(instance) = entry.instance.as_mut() {
                        instance.on_suspend();
                    }

                    entry.suspended = true;
                }
                Lifecycle::Drop => {
                    if let Some(instance) = entry.instance.take() {
                        entry.snapshot = instance.into_snapshot();
                        entry.generation += 1;
                    }
                }
            }
        }

        let entry = &mut self.pages[to];

        if entry.instance.is_none() {
            let mut instance = (entry.factory)(&self.context, &self.registry, to, entry.generation);

            if let Some(snapshot) = entry.snapshot.take() {
                instance.restore(snapshot);
            }

            entry.instance = Some(instance);
        } else if entry.suspended
            && let Some(instance) = entry.instance.as_mut()
        {
            instance.on_resume();
        }

        entry.suspended = false;
        self.current = Some(to);

        if let Some(instance) = entry.instance.as_mut() {
            instance.on_enter();
        }
    }
}

/// The mouse back/forward filter handed to [`event::listen_with`].
// `listen_with` takes `fn(Event, Status, window::Id)`: by value is forced.
#[allow(clippy::needless_pass_by_value)]
fn mouse_navigation(event: Event, status: Status, _window: window::Id) -> Option<RouteMessage> {
    if status == Status::Captured {
        return None;
    }

    match event {
        Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Back)) => {
            Some(RouteMessage::Navigate(Navigation::Back))
        }
        Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Forward)) => {
            Some(RouteMessage::Navigate(Navigation::Forward))
        }
        _ => None,
    }
}

impl<Ctx, Theme, Renderer> std::fmt::Debug for Router<Ctx, Theme, Renderer> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Router")
            .field("pages", &self.pages.len())
            .field("current", &self.current)
            .field("history", &self.history)
            .field("mouse_navigation", &self.mouse_navigation)
            .finish_non_exhaustive()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn press(button: mouse::Button) -> Event {
        Event::Mouse(mouse::Event::ButtonPressed(button))
    }

    #[test]
    fn mouse_filter_honours_capture_and_maps_back_and_forward() {
        let window = window::Id::unique();
        assert!(matches!(
            mouse_navigation(press(mouse::Button::Back), Status::Ignored, window),
            Some(RouteMessage::Navigate(Navigation::Back))
        ));
        assert!(matches!(
            mouse_navigation(press(mouse::Button::Forward), Status::Ignored, window),
            Some(RouteMessage::Navigate(Navigation::Forward))
        ));
        assert!(mouse_navigation(press(mouse::Button::Back), Status::Captured, window).is_none());
        assert!(mouse_navigation(press(mouse::Button::Left), Status::Ignored, window).is_none());
    }
}
