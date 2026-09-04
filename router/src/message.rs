//! The messages a [`Router`](crate::Router) speaks.

use std::any::TypeId;
use std::sync::Arc;

use crate::erased::AnyMessage;
use crate::page::Page;

/// Erased navigation options, built by [`Navigation::to_with`].
///
/// Opaque: a `Payload` can only be created by the router API and is
/// delivered to the target page's
/// [`on_navigate`](crate::Page::on_navigate). Cloning is a reference-count
/// bump.
#[derive(Clone)]
pub struct Payload(Arc<dyn AnyMessage>);

impl Payload {
    pub(crate) fn new<T: Clone + Send + Sync + 'static>(value: T) -> Self {
        Self(Arc::new(value))
    }

    /// A clone of the value if it is a `T`.
    pub(crate) fn downcast<T: Clone + 'static>(&self) -> Option<T> {
        // `(*self.0)`, not `self.0`: the blanket `AnyMessage` impl also
        // covers `Arc<dyn AnyMessage>`, so a call on the `Arc` would erase
        // the `Arc` itself instead of dispatching to the value.
        (*self.0).as_any().downcast_ref::<T>().cloned()
    }

    pub(crate) fn type_name(&self) -> &'static str {
        (*self.0).type_name()
    }
}

// The value itself is deliberately not printed: it may hold secrets.
#[allow(clippy::missing_fields_in_debug)]
impl std::fmt::Debug for Payload {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("Payload").field(&self.type_name()).finish()
    }
}

/// A navigation request.
///
/// Pages return one inside an [`Action`](crate::Action); hosts send one as
/// [`RouteMessage::Navigate`]. The enum and its struct variants are
/// `#[non_exhaustive]`: build values with these constructors:
/// [`to`](Self::to), [`to_with`](Self::to_with), [`index`](Self::index),
/// [`back`](Self::back), [`forward`](Self::forward),
/// [`replace`](Self::replace), and match with a wildcard arm.
///
/// ```
/// use iced_page_router::{Navigation, RouteMessage};
///
/// // A sidebar button that shows page 2:
/// let message = RouteMessage::Navigate(Navigation::index(2));
/// assert!(matches!(message, RouteMessage::Navigate(Navigation::Index(2))));
/// ```
#[derive(Debug, Clone)]
#[non_exhaustive]
pub enum Navigation {
    /// Show the page of type `target` and record it in the history.
    #[non_exhaustive]
    To {
        /// [`TypeId`] of the page type.
        target: TypeId,
        /// [`std::any::type_name`] of the page type; diagnostics only.
        type_name: &'static str,
        /// Options for the page's [`on_navigate`](crate::Page::on_navigate).
        options: Option<Payload>,
    },
    /// Show the page at this index and record it in the history.
    Index(usize),
    /// One step back in the history.
    Back,
    /// One step forward in the history.
    Forward,
    /// Show the page of type `target`, replacing the current history entry.
    #[non_exhaustive]
    Replace {
        /// [`TypeId`] of the page type.
        target: TypeId,
        /// [`std::any::type_name`] of the page type; diagnostics only.
        type_name: &'static str,
    },
}

impl Navigation {
    /// Navigate to page `P`.
    #[must_use]
    pub fn to<P: Page>() -> Self {
        Self::To {
            target: TypeId::of::<P>(),
            type_name: std::any::type_name::<P>(),
            options: None,
        }
    }

    /// Navigate to page `P` with options for its `on_navigate`.
    #[must_use]
    pub fn to_with<P: Page>(options: P::NavigationOptions) -> Self {
        Self::To {
            target: TypeId::of::<P>(),
            type_name: std::any::type_name::<P>(),
            options: Some(Payload::new(options)),
        }
    }

    /// Navigate to the page at `index`.
    #[must_use]
    pub fn index(index: usize) -> Self {
        Self::Index(index)
    }

    /// One step back.
    #[must_use]
    pub fn back() -> Self {
        Self::Back
    }

    /// One step forward.
    #[must_use]
    pub fn forward() -> Self {
        Self::Forward
    }

    /// Navigate to page `P` without adding a history entry.
    #[must_use]
    pub fn replace<P: Page>() -> Self {
        Self::Replace {
            target: TypeId::of::<P>(),
            type_name: std::any::type_name::<P>(),
        }
    }
}

/// A message addressed to one page *instance* of a [`Router`](crate::Router).
///
/// Besides the page index it carries the generation of the instance that
/// produced it. A `Drop` page that is left and later re-created gets a new
/// generation, so a task result from the old instance is dropped instead of
/// being applied to the new one: **a message reaches exactly the instance
/// that produced it, or nobody.**
///
/// The payload is reference-counted; cloning (which iced does freely) does
/// not allocate. Build one with [`Router::message`](crate::Router::message).
#[derive(Clone)]
pub struct PageMessage {
    page_id: usize,
    generation: u64,
    inner: Arc<dyn AnyMessage>,
}

impl PageMessage {
    pub(crate) fn new<M: Clone + Send + Sync + 'static>(
        page_id: usize,
        generation: u64,
        message: M,
    ) -> Self {
        Self {
            page_id,
            generation,
            inner: Arc::new(message),
        }
    }

    /// Index of the page this message is for.
    #[must_use]
    pub fn page_id(&self) -> usize {
        self.page_id
    }

    /// Generation of the page instance this message is for.
    #[must_use]
    pub fn generation(&self) -> u64 {
        self.generation
    }

    /// A clone of the message if it is an `M`.
    pub(crate) fn downcast<M: Clone + 'static>(&self) -> Option<M> {
        // See `Payload::downcast` for why the `Arc` is dereferenced first.
        (*self.inner).as_any().downcast_ref::<M>().cloned()
    }

    pub(crate) fn type_name(&self) -> &'static str {
        (*self.inner).type_name()
    }
}

// `inner` is deliberately reduced to its type name: it may hold secrets.
#[allow(clippy::missing_fields_in_debug)]
impl std::fmt::Debug for PageMessage {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PageMessage")
            .field("page_id", &self.page_id)
            .field("generation", &self.generation)
            .field("message", &self.type_name())
            .finish()
    }
}

/// The message type a [`Router`](crate::Router) speaks.
///
/// Wrap it in the application's message and hand it back to
/// [`Router::update`](crate::Router::update).
#[derive(Debug, Clone)]
#[non_exhaustive]
pub enum RouteMessage {
    /// A message for one page instance.
    Page(PageMessage),
    /// A navigation request.
    Navigate(Navigation),
}

impl From<Navigation> for RouteMessage {
    fn from(navigation: Navigation) -> Self {
        Self::Navigate(navigation)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct Home;

    impl crate::Page for Home {
        type Message = ();
        type NavigationOptions = u8;
        type Context = ();
        // `iced_core` implements `Renderer` for `()` in debug builds only, so
        // the release twins of the `#[should_panic]` tests need a real one.
        type Theme = iced::Theme;
        type Renderer = iced::Renderer;
        fn new((): &(), _: &crate::Registry) -> Self {
            Home
        }
        fn update(&mut self, (): ()) -> crate::Action<()> {
            crate::Action::none()
        }
        fn view(&self) -> iced_core::Element<'_, (), iced::Theme, iced::Renderer> {
            unimplemented!("never rendered")
        }
    }

    #[test]
    fn payload_round_trips_its_type_and_rejects_others() {
        let payload = Payload::new(7u8);
        assert_eq!(payload.downcast::<u8>(), Some(7));
        assert_eq!(payload.downcast::<u16>(), None);
        assert_eq!(payload.type_name(), "u8");
        assert_eq!(format!("{payload:?}"), "Payload(\"u8\")");
    }

    #[test]
    fn page_message_round_trips_and_debugs_without_the_value() {
        let message = PageMessage::new(3, 9, String::from("secret"));
        assert_eq!(message.page_id(), 3);
        assert_eq!(message.generation(), 9);
        assert_eq!(message.downcast::<String>().as_deref(), Some("secret"));
        assert_eq!(message.clone().downcast::<u8>(), None);
        let debug = format!("{message:?}");
        assert!(debug.contains("page_id: 3") && debug.contains("generation: 9"));
        assert!(!debug.contains("secret"));
    }

    #[test]
    fn navigation_constructors_carry_the_target_name() {
        let Navigation::To {
            target,
            type_name,
            options,
        } = Navigation::to_with::<Home>(1u8)
        else {
            panic!("expected To");
        };
        assert_eq!(target, TypeId::of::<Home>());
        assert!(type_name.ends_with("Home"));
        assert_eq!(options.unwrap().downcast::<u8>(), Some(1));
        assert!(matches!(
            Navigation::replace::<Home>(),
            Navigation::Replace { .. }
        ));
        assert!(matches!(
            RouteMessage::from(Navigation::back()),
            RouteMessage::Navigate(Navigation::Back)
        ));
    }

    #[test]
    fn route_message_is_send_and_sync() {
        fn assert_send_sync<T: Send + Sync>() {}
        assert_send_sync::<RouteMessage>();
    }
}
