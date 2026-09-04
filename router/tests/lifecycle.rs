//! Router behaviour that a page author relies on. One message enum per page
//! (the crate's whole point is per-page message types).

use std::any::Any;
use std::task::{Context, Poll, Waker};

use iced::widget::text;
use iced::{Element, Subscription, Task};
use iced_page_router::{
    Action, Key, Lifecycle, Navigation, NavigationError, Page, Registry, RouteMessage, Router,
    Shared,
};

struct Ctx;

type TestRouter = Router<Ctx, iced::Theme, iced::Renderer>;

/// Mirrors `Counter::n` so tests can observe it from outside.
struct Seen;
impl Key for Seen {
    type Value = u32;
}

/// Lifecycle calls recorded by `Kept`.
struct Log;
impl Key for Log {
    type Value = Vec<String>;
}

/// Set by `Always::new`.
struct Built;
impl Key for Built {
    type Value = bool;
}

/// Runs a task to completion on the current thread and collects its output.
///
/// `Task::stream` starts with one `yield_now`, so the first poll is
/// `Pending`; a noop waker and a bounded re-poll loop are enough.
fn drain<T>(task: Task<T>) -> Vec<T> {
    let Some(mut stream) = iced_runtime::task::into_stream(task) else {
        return Vec::new();
    };
    let mut cx = Context::from_waker(Waker::noop());
    let mut out = Vec::new();

    for _ in 0..1_000 {
        match stream.as_mut().poll_next(&mut cx) {
            Poll::Ready(Some(iced_runtime::Action::Output(value))) => out.push(value),
            Poll::Ready(Some(_)) | Poll::Pending => {}
            Poll::Ready(None) => return out,
        }
    }

    panic!("task did not finish within 1000 polls");
}

/// A stream that never yields: enough to count subscription units.
fn never<M>() -> iced::futures::stream::Pending<M> {
    iced::futures::stream::pending()
}

// ---------------------------------------------------------------- pages --

#[derive(Debug, Clone)]
enum CounterMessage {
    Bump,
    /// Navigate to `Kept` with options.
    GoKept,
    /// A task plus a navigation away from this `Drop` page.
    TaskThenKept,
    /// A task whose result comes back to this very instance.
    BumpViaTask,
}

/// A `Drop` page whose count survives through `into_snapshot`/`restore`.
struct Counter {
    n: u32,
    seen: Shared<u32>,
}

impl Page for Counter {
    type Message = CounterMessage;
    type NavigationOptions = ();
    type Context = Ctx;
    type Theme = iced::Theme;
    type Renderer = iced::Renderer;

    fn new(_: &Ctx, registry: &Registry) -> Self {
        Self {
            n: 0,
            seen: registry.get_or_insert_with::<Seen>(|| 0),
        }
    }

    fn update(&mut self, message: CounterMessage) -> Action<CounterMessage> {
        match message {
            CounterMessage::Bump => {
                self.n += 1;
                self.seen.set(self.n);
                Action::none()
            }
            CounterMessage::GoKept => Action::navigate_with::<Kept>(7),
            CounterMessage::TaskThenKept => Action::task(Task::done(CounterMessage::Bump))
                .and_navigate(Navigation::to_with::<Kept>(1)),
            CounterMessage::BumpViaTask => Action::task(Task::done(CounterMessage::Bump)),
        }
    }

    fn view(&self) -> Element<'_, CounterMessage> {
        text(self.n).into()
    }

    fn into_snapshot(self) -> Option<Box<dyn Any>> {
        Some(Box::new(self.n))
    }

    fn restore(&mut self, snapshot: Box<dyn Any>) {
        self.n = *snapshot.downcast::<u32>().expect("our own snapshot");
        self.seen.set(self.n);
    }
}

#[derive(Debug, Clone)]
enum KeptMessage {}

/// A `Suspend` page that records its lifecycle calls.
struct Kept {
    log: Shared<Vec<String>>,
}

impl Page for Kept {
    type Message = KeptMessage;
    type NavigationOptions = u8;
    type Context = Ctx;
    type Theme = iced::Theme;
    type Renderer = iced::Renderer;

    const LIFECYCLE: Lifecycle = Lifecycle::Suspend;

    fn new(_: &Ctx, registry: &Registry) -> Self {
        Self {
            log: registry.get_or_insert_with::<Log>(Vec::new),
        }
    }

    fn update(&mut self, message: KeptMessage) -> Action<KeptMessage> {
        match message {}
    }

    fn view(&self) -> Element<'_, KeptMessage> {
        text("kept").into()
    }

    /// One unit, only while current.
    fn subscription(&self) -> Subscription<KeptMessage> {
        Subscription::run(never::<KeptMessage>)
    }

    fn on_enter(&mut self) {
        self.log.update(|log| log.push("enter".into()));
    }

    fn on_navigate(&mut self, options: u8) {
        self.log
            .update(|log| log.push(format!("navigate {options}")));
    }

    fn on_suspend(&mut self) {
        self.log.update(|log| log.push("suspend".into()));
    }

    fn on_resume(&mut self) {
        self.log.update(|log| log.push("resume".into()));
    }
}

#[derive(Debug, Clone)]
enum AlwaysMessage {}

/// A `Resident` page.
struct Always;

impl Page for Always {
    type Message = AlwaysMessage;
    type NavigationOptions = ();
    type Context = Ctx;
    type Theme = iced::Theme;
    type Renderer = iced::Renderer;

    const LIFECYCLE: Lifecycle = Lifecycle::Resident;

    fn new(_: &Ctx, registry: &Registry) -> Self {
        registry.get_or_insert_with::<Built>(|| false).set(true);
        Self
    }

    fn update(&mut self, message: AlwaysMessage) -> Action<AlwaysMessage> {
        match message {}
    }

    fn view(&self) -> Element<'_, AlwaysMessage> {
        text("always").into()
    }

    /// One unit for as long as the instance exists, i.e. always.
    fn background_subscription(&self) -> Subscription<AlwaysMessage> {
        Subscription::run(never::<AlwaysMessage>)
    }
}

#[derive(Debug, Clone)]
enum GhostMessage {}

/// Never added to any router.
struct Ghost;

impl Page for Ghost {
    type Message = GhostMessage;
    type NavigationOptions = ();
    type Context = Ctx;
    type Theme = iced::Theme;
    type Renderer = iced::Renderer;

    fn new(_: &Ctx, _: &Registry) -> Self {
        Self
    }

    fn update(&mut self, message: GhostMessage) -> Action<GhostMessage> {
        match message {}
    }

    fn view(&self) -> Element<'_, GhostMessage> {
        text("").into()
    }
}

#[derive(Debug, Clone)]
enum OuterMessage {
    Inner(RouteMessage),
}

/// A page that owns its own router.
struct Outer {
    inner: TestRouter,
}

impl Page for Outer {
    type Message = OuterMessage;
    type NavigationOptions = ();
    type Context = Ctx;
    type Theme = iced::Theme;
    type Renderer = iced::Renderer;

    fn new(_: &Ctx, registry: &Registry) -> Self {
        let mut inner = Router::new(registry.clone(), Ctx).mouse_navigation(false);
        inner.add::<Counter>("inner counter");
        inner.navigate::<Counter>().unwrap();
        Self { inner }
    }

    fn update(&mut self, message: OuterMessage) -> Action<OuterMessage> {
        match message {
            OuterMessage::Inner(inner) => {
                Action::task(self.inner.update(inner).map(OuterMessage::Inner))
            }
        }
    }

    fn view(&self) -> Element<'_, OuterMessage> {
        self.inner
            .view()
            .map_or_else(|| text("").into(), |page| page.map(OuterMessage::Inner))
    }
}

fn router() -> (TestRouter, Registry) {
    let registry = Registry::new();
    let mut router = Router::new(registry.clone(), Ctx);
    router.add::<Counter>("counter").add::<Kept>("kept");
    (router, registry)
}

fn seen(registry: &Registry) -> u32 {
    registry.get::<Seen>().map_or(0, |seen| seen.get())
}

fn log(registry: &Registry) -> Vec<String> {
    registry.get::<Log>().map_or_else(Vec::new, |log| log.get())
}

fn bump(router: &mut TestRouter) {
    let message = router
        .message::<Counter>(CounterMessage::Bump)
        .expect("Counter is live");
    let _ = router.update(message);
}

// ---------------------------------------------------------- lifecycle --

#[test]
fn a_drop_page_is_restored_from_its_snapshot_on_navigate() {
    let (mut router, registry) = router();
    router.navigate::<Counter>().unwrap();
    bump(&mut router);
    bump(&mut router);
    assert_eq!(seen(&registry), 2);

    router.navigate::<Kept>().unwrap();
    assert!(
        router.page::<Counter>().is_none(),
        "the instance was dropped"
    );
    registry.get::<Seen>().unwrap().set(0);
    router.navigate::<Counter>().unwrap();
    assert_eq!(seen(&registry), 2, "restore() put the snapshot back");
    assert_eq!(router.page::<Counter>().unwrap().n, 2);
}

#[test]
fn back_restores_a_drop_page_from_its_snapshot() {
    let (mut router, registry) = router();
    router.navigate::<Counter>().unwrap();
    bump(&mut router);
    router.navigate::<Kept>().unwrap();
    registry.get::<Seen>().unwrap().set(0);
    assert_eq!(router.back(), Some(0));
    assert_eq!(seen(&registry), 1);
    assert_eq!(router.forward(), Some(1));
    assert_eq!(router.forward(), None);
}

#[test]
fn a_suspend_page_gets_every_lifecycle_call_in_order() {
    let (mut router, registry) = router();
    router.navigate::<Counter>().unwrap();
    let go = router.message::<Counter>(CounterMessage::GoKept).unwrap();
    let _ = router.update(go);
    assert_eq!(router.current(), Some(1));
    router.navigate::<Counter>().unwrap();
    router.navigate::<Kept>().unwrap();
    assert_eq!(router.back(), Some(0));
    assert_eq!(router.forward(), Some(1));

    assert_eq!(
        log(&registry),
        vec![
            "enter",
            "navigate 7",
            "suspend",
            "resume",
            "enter",
            "suspend",
            "resume",
            "enter",
        ]
    );
}

#[test]
fn navigating_to_the_current_page_delivers_options_but_not_on_enter() {
    let (mut router, registry) = router();
    router.navigate_with::<Kept>(3).unwrap();
    router.navigate_with::<Kept>(4).unwrap();
    assert_eq!(log(&registry), vec!["enter", "navigate 3", "navigate 4"]);
}

#[test]
fn a_resident_page_exists_before_its_first_visit_and_survives_leaving() {
    let registry = Registry::new();
    let mut router: TestRouter = Router::new(registry.clone(), Ctx);
    router.add::<Always>("always").add::<Counter>("counter");
    assert!(registry.get::<Built>().unwrap().get(), "constructed at add");
    assert!(router.page::<Always>().is_some());
    assert_eq!(router.current(), None);

    router.navigate::<Always>().unwrap();
    router.navigate::<Counter>().unwrap();
    assert!(router.page::<Always>().is_some(), "never dropped");
    assert_eq!(
        router.pages().map(|p| p.lifecycle).collect::<Vec<_>>(),
        vec![Lifecycle::Resident, Lifecycle::Drop]
    );
}

// ------------------------------------------------------------ history --

#[test]
fn navigating_to_the_current_page_does_not_grow_history() {
    let (mut router, _) = router();
    router.navigate::<Counter>().unwrap();
    router.navigate::<Counter>().unwrap();
    router.navigate::<Kept>().unwrap();

    let _ = router.update(RouteMessage::Navigate(Navigation::back()));
    assert_eq!(router.current(), Some(0));
    let _ = router.update(RouteMessage::Navigate(Navigation::back()));
    assert_eq!(
        router.current(),
        Some(0),
        "one back is enough: no duplicate entries"
    );
    let _ = router.update(RouteMessage::Navigate(Navigation::forward()));
    assert_eq!(router.current(), Some(1));
}

#[test]
fn navigating_to_the_cursor_page_after_back_truncates_the_forward_branch() {
    let (mut router, _) = router();
    router.navigate::<Counter>().unwrap();
    router.navigate::<Kept>().unwrap();
    assert_eq!(router.back(), Some(0));
    assert!(router.can_go_forward());
    router.navigate::<Counter>().unwrap();
    assert!(
        !router.can_go_forward(),
        "browser semantics: forward branch discarded"
    );
    assert_eq!(router.forward(), None);
}

#[test]
fn history_is_bounded_through_the_router() {
    let registry = Registry::new();
    let mut router: TestRouter = Router::new(registry, Ctx).history_len(2);
    router.add::<Counter>("counter").add::<Kept>("kept");
    router.navigate::<Counter>().unwrap();
    router.navigate::<Kept>().unwrap();
    router.navigate::<Counter>().unwrap();
    assert_eq!(router.back(), Some(1));
    assert_eq!(router.back(), None, "the oldest entry was forgotten");
}

#[test]
fn replace_swaps_the_history_entry() {
    let (mut router, _) = router();
    router.navigate::<Counter>().unwrap();
    assert_eq!(router.replace::<Kept>(), Ok(1));
    assert_eq!(router.current(), Some(1));
    assert!(!router.can_go_back());
    let _ = router.update(RouteMessage::Navigate(Navigation::replace::<Counter>()));
    assert_eq!(router.current(), Some(0));
    assert!(!router.can_go_back());
}

#[test]
fn can_go_back_and_forward_follow_the_cursor() {
    let (mut router, _) = router();
    assert!(!router.can_go_back() && !router.can_go_forward());
    router.navigate::<Counter>().unwrap();
    router.navigate::<Kept>().unwrap();
    assert!(router.can_go_back() && !router.can_go_forward());
    assert_eq!(router.back(), Some(0));
    assert!(!router.can_go_back() && router.can_go_forward());
    assert_eq!(router.back(), None);
    assert_eq!(router.forward(), Some(1));
    assert_eq!(router.forward(), None);
}

#[test]
fn navigate_index_through_a_message() {
    let (mut router, _) = router();
    let _ = router.update(RouteMessage::Navigate(Navigation::index(1)));
    assert_eq!(router.current(), Some(1));
    assert_eq!(router.navigate_index(0), Ok(0));
}

// ------------------------------------------------------------- errors --

#[test]
fn unknown_targets_are_errors_not_silence() {
    let (mut router, _) = router();
    let unknown = NavigationError::UnknownPage {
        type_name: std::any::type_name::<Ghost>(),
    };
    assert_eq!(router.navigate::<Ghost>(), Err(unknown.clone()));
    assert_eq!(router.replace::<Ghost>(), Err(unknown));
    assert_eq!(
        router.navigate_index(99),
        Err(NavigationError::IndexOutOfRange { index: 99, len: 2 })
    );
    assert_eq!(router.current(), None);
    assert!(router.view().is_none());
}

#[test]
#[cfg(debug_assertions)]
#[should_panic(expected = "not in this router")]
fn a_navigation_to_an_unknown_page_from_update_is_a_programming_error() {
    let (mut router, _) = router();
    let _ = router.update(RouteMessage::Navigate(Navigation::to::<Ghost>()));
}

#[test]
#[cfg(not(debug_assertions))]
fn a_navigation_to_an_unknown_page_from_update_is_ignored_in_release() {
    let (mut router, _) = router();
    router.navigate::<Counter>().unwrap();
    let _ = router.update(RouteMessage::Navigate(Navigation::to::<Ghost>()));
    let _ = router.update(RouteMessage::Navigate(Navigation::index(99)));
    assert_eq!(router.current(), Some(0));
}

#[test]
#[cfg(debug_assertions)]
#[should_panic(expected = "not in this router")]
fn a_replace_with_an_unknown_page_from_update_is_a_programming_error() {
    let (mut router, _) = router();
    router.navigate::<Counter>().unwrap();
    let _ = router.update(RouteMessage::Navigate(Navigation::replace::<Ghost>()));
}

#[test]
#[cfg(not(debug_assertions))]
fn a_replace_with_an_unknown_page_from_update_is_ignored_in_release() {
    let (mut router, _) = router();
    router.navigate::<Counter>().unwrap();
    let _ = router.update(RouteMessage::Navigate(Navigation::replace::<Ghost>()));
    assert_eq!(router.current(), Some(0));
    assert!(!router.can_go_back());
}

#[test]
#[cfg(debug_assertions)]
#[should_panic(expected = "added twice")]
fn adding_a_page_twice_is_a_programming_error() {
    let (mut router, _) = router();
    router.add::<Counter>("again");
}

#[test]
#[cfg(not(debug_assertions))]
fn adding_a_page_twice_is_ignored_in_release() {
    let (mut router, _) = router();
    let before: Vec<_> = router.pages().map(|p| p.name.to_owned()).collect();
    router.add::<Counter>("again");
    let after: Vec<_> = router.pages().map(|p| p.name.to_owned()).collect();
    assert_eq!(after, before, "the existing entry stays");
    assert_eq!(router.index_of::<Counter>(), Some(0));
}

#[test]
#[cfg(debug_assertions)]
#[should_panic(expected = "at least one entry")]
fn a_zero_history_length_is_a_programming_error() {
    let _: TestRouter = Router::new(Registry::new(), Ctx).history_len(0);
}

#[test]
#[cfg(not(debug_assertions))]
fn a_zero_history_length_keeps_one_entry_in_release() {
    let (router, _) = router();
    let mut router = router.history_len(0);
    router.navigate::<Counter>().unwrap();
    router.navigate::<Kept>().unwrap();
    assert_eq!(router.current(), Some(1));
    assert!(!router.can_go_back());
}

// ---------------------------------------------------------- messages --

#[test]
fn message_is_typed_and_none_without_a_live_instance() {
    let (mut router, _) = router();
    assert!(router.message::<Counter>(CounterMessage::Bump).is_none());
    router.navigate::<Counter>().unwrap();
    let message = router.message::<Counter>(CounterMessage::Bump).unwrap();
    let RouteMessage::Page(page) = &message else {
        panic!("expected a page message");
    };
    assert_eq!((page.page_id(), page.generation()), (0, 0));
}

#[test]
fn a_message_for_a_page_without_an_instance_is_dropped() {
    let (mut router, registry) = router();
    router.navigate::<Counter>().unwrap();
    let message = router.message::<Counter>(CounterMessage::Bump).unwrap();
    router.navigate::<Kept>().unwrap();
    let _ = router.update(message);
    assert_eq!(
        seen(&registry),
        0,
        "no instance, nothing to update, no panic"
    );
}

#[test]
fn a_late_task_result_from_a_dropped_instance_is_dropped() {
    let (mut router, registry) = router();
    router.navigate::<Counter>().unwrap();
    let start = router
        .message::<Counter>(CounterMessage::TaskThenKept)
        .unwrap();
    let task = router.update(start);
    assert_eq!(router.current(), Some(1), "the navigation happened");

    let late: Vec<RouteMessage> = drain(task);
    assert_eq!(late.len(), 1, "the router mapped the page's task");
    let RouteMessage::Page(page) = &late[0] else {
        panic!("expected a page message");
    };
    assert_eq!((page.page_id(), page.generation()), (0, 0));

    // Back to Counter: a *new* instance (generation 1) restored from the
    // snapshot. The stale result must not touch it.
    router.navigate::<Counter>().unwrap();
    for message in late {
        let _ = router.update(message);
    }
    assert_eq!(
        seen(&registry),
        0,
        "the re-created instance never saw the result"
    );
    assert_eq!(router.page::<Counter>().unwrap().n, 0);

    let fresh = router.message::<Counter>(CounterMessage::Bump).unwrap();
    let RouteMessage::Page(page) = &fresh else {
        panic!("expected a page message");
    };
    assert_eq!(page.generation(), 1, "new instance, new generation");
}

#[test]
fn a_task_result_for_a_live_instance_is_delivered() {
    let (mut router, registry) = router();
    router.navigate::<Counter>().unwrap();
    let start = router
        .message::<Counter>(CounterMessage::BumpViaTask)
        .unwrap();
    let results = drain(router.update(start));
    assert_eq!(results.len(), 1);
    for message in results {
        let _ = router.update(message);
    }
    assert_eq!(seen(&registry), 1);
}

#[test]
#[cfg(debug_assertions)]
#[should_panic(expected = "which expects")]
fn a_message_from_another_router_is_a_programming_error() {
    let registry = Registry::new();
    let mut other: TestRouter = Router::new(registry.clone(), Ctx);
    other.add::<Outer>("outer");
    other.navigate::<Outer>().unwrap();
    let foreign = other
        .message::<Outer>(OuterMessage::Inner(RouteMessage::Navigate(
            Navigation::back(),
        )))
        .unwrap();

    let (mut router, _) = router();
    router.navigate::<Counter>().unwrap();
    // Same index (0), same generation (0), wrong message type.
    let _ = router.update(foreign);
}

#[test]
#[cfg(not(debug_assertions))]
fn a_message_from_another_router_is_dropped_in_release() {
    let registry = Registry::new();
    let mut other: TestRouter = Router::new(registry.clone(), Ctx);
    other.add::<Outer>("outer");
    other.navigate::<Outer>().unwrap();
    let foreign = other
        .message::<Outer>(OuterMessage::Inner(RouteMessage::Navigate(
            Navigation::back(),
        )))
        .unwrap();

    let (mut router, registry) = router();
    router.navigate::<Counter>().unwrap();
    let _ = router.update(foreign);
    assert_eq!(seen(&registry), 0);
}

#[test]
fn nested_routers_route_independently() {
    let registry = Registry::new();
    let mut outer: TestRouter = Router::new(registry.clone(), Ctx);
    outer.add::<Outer>("outer");
    outer.navigate::<Outer>().unwrap();

    // A message for the inner counter travels outer -> Outer page -> inner.
    let inner_bump = outer
        .page::<Outer>()
        .unwrap()
        .inner
        .message::<Counter>(CounterMessage::Bump)
        .unwrap();
    let wrapped = outer
        .message::<Outer>(OuterMessage::Inner(inner_bump))
        .unwrap();
    let _ = outer.update(wrapped);
    assert_eq!(seen(&registry), 1);
    assert_eq!(outer.page::<Outer>().unwrap().inner.current(), Some(0));
}

// ------------------------------------------------------ subscriptions --

#[test]
fn subscription_composes_background_of_every_instance_and_current_only() {
    let registry = Registry::new();
    let mut router: TestRouter = Router::new(registry, Ctx).mouse_navigation(false);
    router
        .add::<Always>("always")
        .add::<Counter>("counter")
        .add::<Kept>("kept");
    assert_eq!(
        router.subscription().units(),
        1,
        "the Resident page's background subscription runs before any visit"
    );

    router.navigate::<Counter>().unwrap();
    assert_eq!(
        router.subscription().units(),
        1,
        "Counter has no subscription"
    );

    router.navigate::<Kept>().unwrap();
    assert_eq!(
        router.subscription().units(),
        2,
        "Always (background, not current) + Kept (current)"
    );

    router.navigate::<Counter>().unwrap();
    assert_eq!(
        router.subscription().units(),
        1,
        "a suspended Kept keeps its instance but not its `subscription`"
    );

    router.navigate::<Always>().unwrap();
    assert_eq!(
        router.subscription().units(),
        1,
        "background once, no double count"
    );
}

#[test]
fn mouse_navigation_adds_one_listener() {
    let (with_mouse, _) = router();
    assert_eq!(
        with_mouse.subscription().units(),
        1,
        "the mouse listener only"
    );
    let (without_mouse, _) = router();
    let without_mouse = without_mouse.mouse_navigation(false);
    assert_eq!(without_mouse.subscription().units(), 0);
}

// ---------------------------------------------------------- inspection --

#[test]
fn pages_index_of_page_and_page_mut() {
    let (mut router, _) = router();
    assert_eq!(router.index_of::<Counter>(), Some(0));
    assert_eq!(router.index_of::<Kept>(), Some(1));
    assert_eq!(router.index_of::<Ghost>(), None);

    router.navigate::<Counter>().unwrap();
    let infos: Vec<(usize, &str, bool)> = router
        .pages()
        .map(|p| (p.index, p.name, p.is_current))
        .collect();
    assert_eq!(infos, vec![(0, "counter", true), (1, "kept", false)]);

    router.page_mut::<Counter>().unwrap().n = 41;
    assert_eq!(router.page::<Counter>().unwrap().n, 41);
    assert!(router.page::<Kept>().is_none(), "not built yet");
    assert!(router.page::<Ghost>().is_none());
}

#[test]
fn context_and_registry_are_reachable() {
    struct Counted(u32);
    let registry = Registry::new();
    let mut router: Router<Counted, iced::Theme, iced::Renderer> =
        Router::new(registry.clone(), Counted(1));
    router.context_mut().0 += 1;
    assert_eq!(router.context().0, 2);
    let _ = router.registry().insert::<Seen>(5).unwrap();
    assert_eq!(registry.get::<Seen>().unwrap().get(), 5);
}

#[test]
fn debug_impls_mention_state_but_not_payloads() {
    let (mut router, _) = router();
    router.navigate::<Counter>().unwrap();
    let debug = format!("{router:?}");
    assert!(debug.contains("pages: 2") && debug.contains("current: Some(0)"));

    let message = router.message::<Counter>(CounterMessage::Bump).unwrap();
    let debug = format!("{message:?}");
    assert!(debug.contains("Page(PageMessage") && debug.contains("CounterMessage"));

    let navigation = format!("{:?}", Navigation::to_with::<Kept>(9));
    assert!(navigation.contains("Kept") && navigation.contains("Payload(\"u8\")"));

    let action: Action<CounterMessage> = Action::back();
    assert!(format!("{action:?}").contains("Back"));

    let info = router.pages().next().unwrap();
    assert!(format!("{info:?}").contains("counter"));
}
