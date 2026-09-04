//! What a page's `update` returns.

use iced_runtime::Task;
use iced_runtime::futures::MaybeSend;

use crate::message::Navigation;
use crate::page::Page;

/// A task to run and, optionally, a navigation to perform.
///
/// Mirrors the subset of [`Task`] a page needs, plus navigation:
///
/// ```
/// use iced::Task;
/// use iced_page_router::{Action, Navigation};
///
/// # struct Home; struct Settings;
/// # impl iced_page_router::Page for Home {
/// #     type Message = (); type NavigationOptions = (); type Context = ();
/// #     type Theme = iced::Theme; type Renderer = iced::Renderer;
/// #     fn new(_: &(), _: &iced_page_router::Registry) -> Self { Home }
/// #     fn update(&mut self, _: ()) -> Action<()> { Action::none() }
/// #     fn view(&self) -> iced::Element<'_, ()> { iced::widget::text("").into() }
/// # }
/// # impl iced_page_router::Page for Settings {
/// #     type Message = (); type NavigationOptions = bool; type Context = ();
/// #     type Theme = iced::Theme; type Renderer = iced::Renderer;
/// #     fn new(_: &(), _: &iced_page_router::Registry) -> Self { Settings }
/// #     fn update(&mut self, _: ()) -> Action<()> { Action::none() }
/// #     fn view(&self) -> iced::Element<'_, ()> { iced::widget::text("").into() }
/// # }
/// #[derive(Clone)]
/// enum Message { Saved, Cancel }
///
/// let _go: Action<Message> = Action::navigate::<Home>();
/// let _go_with: Action<Message> = Action::navigate_with::<Settings>(true);
/// let _cancel: Action<Message> = Action::back();
/// let _save_then_go: Action<Message> =
///     Action::task(Task::done(Message::Saved)).and_navigate(Navigation::to::<Home>());
/// let _from_task: Action<Message> = Task::done(Message::Cancel).into();
/// ```
///
/// The router applies the navigation inside `Router::update`, *before* the
/// returned task is handed to iced and started, so no result of the task
/// can arrive earlier than the navigation. See [`Page`] for what happens
/// to a task whose page is dropped by it.
#[must_use]
pub struct Action<M> {
    pub(crate) task: Task<M>,
    pub(crate) navigation: Option<Navigation>,
}

impl<M> Action<M> {
    /// Nothing to do.
    pub fn none() -> Self {
        Self {
            task: Task::none(),
            navigation: None,
        }
    }

    /// Run `task`.
    pub fn task(task: Task<M>) -> Self {
        Self {
            task,
            navigation: None,
        }
    }

    /// Navigate to `P` (a page added to the same router).
    pub fn navigate<P: Page>() -> Self {
        Self::none().and_navigate(Navigation::to::<P>())
    }

    /// Navigate to `P` with options for its [`Page::on_navigate`].
    pub fn navigate_with<P: Page>(options: P::NavigationOptions) -> Self {
        Self::none().and_navigate(Navigation::to_with::<P>(options))
    }

    /// One step back in the history.
    pub fn back() -> Self {
        Self::none().and_navigate(Navigation::back())
    }

    /// One step forward in the history.
    pub fn forward() -> Self {
        Self::none().and_navigate(Navigation::forward())
    }

    /// Navigate to `P` without adding a history entry (login → home).
    pub fn replace<P: Page>() -> Self {
        Self::none().and_navigate(Navigation::replace::<P>())
    }

    /// Also run `task`.
    pub fn and_task(self, task: Task<M>) -> Self
    where
        M: 'static,
    {
        Self {
            task: Task::batch([self.task, task]),
            navigation: self.navigation,
        }
    }

    /// Also perform `navigation` (replacing any navigation set earlier).
    pub fn and_navigate(mut self, navigation: Navigation) -> Self {
        self.navigation = Some(navigation);
        self
    }

    /// Maps the task's messages so a component action can be used by a page.
    pub fn map<O>(self, f: impl Fn(M) -> O + MaybeSend + 'static) -> Action<O>
    where
        M: MaybeSend + 'static,
        O: MaybeSend + 'static,
    {
        Action {
            task: self.task.map(f),
            navigation: self.navigation,
        }
    }
}

impl<M> From<Task<M>> for Action<M> {
    fn from(task: Task<M>) -> Self {
        Self::task(task)
    }
}

impl<M> std::fmt::Debug for Action<M> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Action")
            .field("task_units", &self.task.units())
            .field("navigation", &self.navigation)
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn constructors_set_the_navigation_and_keep_the_task() {
        let action: Action<u8> = Action::task(Task::done(1)).and_navigate(Navigation::back());
        assert!(matches!(action.navigation, Some(Navigation::Back)));
        assert_eq!(action.task.units(), 1);

        let action: Action<u8> = Action::from(Task::done(1)).and_task(Task::done(2));
        assert_eq!(action.task.units(), 2);
        assert!(action.navigation.is_none());

        let mapped: Action<u16> = Action::<u8>::forward().map(u16::from);
        assert!(matches!(mapped.navigation, Some(Navigation::Forward)));
        assert_eq!(mapped.task.units(), 0);
        assert!(format!("{mapped:?}").contains("Forward"));
    }
}
