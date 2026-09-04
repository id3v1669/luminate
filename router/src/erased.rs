//! Type erasure: messages/options (`AnyMessage`) and pages (`ErasedPage`).

use std::any::Any;

use iced_core::Element;
use iced_runtime::Task;
use iced_runtime::futures::Subscription;

use crate::message::{Navigation, PageMessage, Payload, RouteMessage};
use crate::page::Page;

/// A page message or navigation payload with its concrete type erased.
///
/// `Send + Sync` so that an `Arc<dyn AnyMessage>` is `Send`, every
/// `RouteMessage` must be, because iced's `Task::map` and `Program::Message`
/// require it.
pub(crate) trait AnyMessage: Any + Send + Sync {
    fn as_any(&self) -> &dyn Any;
    fn type_name(&self) -> &'static str;
}

impl<T: Any + Send + Sync> AnyMessage for T {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn type_name(&self) -> &'static str {
        std::any::type_name::<T>()
    }
}

/// The programming-error policy (CONTRIBUTING.md): log at `error`, and
/// panic in debug builds so the mistake is noticed during development.
/// Release builds degrade as documented at each call site.
#[track_caller]
pub(crate) fn programming_error(message: std::fmt::Arguments<'_>) {
    log::error!("{message}");

    #[cfg(debug_assertions)]
    panic!("{message}");
}

/// Type-erased page operations. One implementor: [`Erased<P>`].
pub(crate) trait ErasedPage<Theme, Renderer> {
    /// Generation of this instance, stamped into every message it emits.
    fn generation(&self) -> u64;
    fn update(&mut self, message: &PageMessage) -> (Task<RouteMessage>, Option<Navigation>);
    fn subscription(&self) -> Subscription<RouteMessage>;
    fn background_subscription(&self) -> Subscription<RouteMessage>;
    fn view(&self) -> Element<'_, RouteMessage, Theme, Renderer>;
    fn on_enter(&mut self);
    fn on_navigate(&mut self, options: &Payload);
    fn on_suspend(&mut self);
    fn on_resume(&mut self);
    fn restore(&mut self, snapshot: Box<dyn Any>);
    fn into_snapshot(self: Box<Self>) -> Option<Box<dyn Any>>;
    fn as_any(&self) -> &dyn Any;
    fn as_any_mut(&mut self) -> &mut dyn Any;
}

/// A page plus the identity the router gave its instance.
pub(crate) struct Erased<P> {
    pub(crate) page: P,
    id: usize,
    generation: u64,
}

impl<P: Page> Erased<P> {
    pub(crate) fn new(page: P, id: usize, generation: u64) -> Self {
        Self {
            page,
            id,
            generation,
        }
    }

    fn stamp(id: usize, generation: u64, message: P::Message) -> RouteMessage {
        RouteMessage::Page(PageMessage::new(id, generation, message))
    }
}

impl<P: Page> ErasedPage<P::Theme, P::Renderer> for Erased<P> {
    fn generation(&self) -> u64 {
        self.generation
    }

    fn update(&mut self, message: &PageMessage) -> (Task<RouteMessage>, Option<Navigation>) {
        let Some(message) = message.downcast::<P::Message>() else {
            programming_error(format_args!(
                "message of type {} delivered to page {} ({}), which expects {}; dropped",
                message.type_name(),
                self.id,
                std::any::type_name::<P>(),
                std::any::type_name::<P::Message>()
            ));
            return (Task::none(), None);
        };

        let action = self.page.update(message);
        let (id, generation) = (self.id, self.generation);
        let task = action.task.map(move |m| Self::stamp(id, generation, m));

        (task, action.navigation)
    }

    fn subscription(&self) -> Subscription<RouteMessage> {
        self.page
            .subscription()
            .with((self.id, self.generation))
            .map(|((id, generation), m)| Self::stamp(id, generation, m))
    }

    fn background_subscription(&self) -> Subscription<RouteMessage> {
        self.page
            .background_subscription()
            .with((self.id, self.generation))
            .map(|((id, generation), m)| Self::stamp(id, generation, m))
    }

    fn view(&self) -> Element<'_, RouteMessage, P::Theme, P::Renderer> {
        let (id, generation) = (self.id, self.generation);
        self.page
            .view()
            .map(move |m| Self::stamp(id, generation, m))
    }

    fn on_enter(&mut self) {
        self.page.on_enter();
    }

    fn on_navigate(&mut self, options: &Payload) {
        match options.downcast::<P::NavigationOptions>() {
            Some(options) => self.page.on_navigate(options),
            None => programming_error(format_args!(
                "navigation options of type {} for page {}, which expects {}; ignored",
                options.type_name(),
                std::any::type_name::<P>(),
                std::any::type_name::<P::NavigationOptions>()
            )),
        }
    }

    fn on_suspend(&mut self) {
        self.page.on_suspend();
    }

    fn on_resume(&mut self) {
        self.page.on_resume();
    }

    fn restore(&mut self, snapshot: Box<dyn Any>) {
        self.page.restore(snapshot);
    }

    fn into_snapshot(self: Box<Self>) -> Option<Box<dyn Any>> {
        self.page.into_snapshot()
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}
