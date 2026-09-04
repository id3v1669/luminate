//! Text inputs, `NavigationOptions`, `Shared` state and `Lifecycle::Suspend`.
//!
//! This page is **suspended**, not dropped, when the user leaves it: the
//! instance stays alive, `on_suspend`/`on_resume` run, and `on_resume`
//! re-reads the draft that the nested sidebar's copy of this page may have
//! edited through the same [`Registry`] entry. Compare
//! [`snapshot`](crate::pages::snapshot), which is dropped and restored.

use iced_luminate::descriptor::{Button, Input};
use iced_luminate::iced::widget::{Space, column};
use iced_luminate::router::{Action, Key, Lifecycle, Page, Registry, Shared};
use iced_luminate::{Element, Luminate, Renderer, Theme};

/// The registry key of the draft every inputs page shares.
pub(crate) struct Draft;

impl Key for Draft {
    type Value = String;
}

/// Messages of the inputs page.
#[derive(Debug, Clone)]
pub(crate) enum Message {
    /// The draft changed.
    InputChanged(String),
    /// Unlock the input again.
    Unlock,
}

/// What a navigation to this page may carry.
#[derive(Debug, Clone, Copy)]
pub(crate) struct Options {
    /// Arrive with the input locked (read-only) until "Unlock" is pressed.
    pub(crate) locked: bool,
}

/// One input bound to a shared draft.
pub(crate) struct InputsPage {
    luminate: Luminate,
    locked: bool,
    draft: String,
    shared: Shared<String>,
}

impl Page for InputsPage {
    type Message = Message;
    type NavigationOptions = Options;
    type Context = Luminate;
    type Theme = Theme;
    type Renderer = Renderer;

    /// Keep the instance while another page is shown.
    const LIFECYCLE: Lifecycle = Lifecycle::Suspend;

    fn new(luminate: &Luminate, registry: &Registry) -> Self {
        let shared = registry.get_or_insert_with::<Draft>(String::new);
        let draft = shared.get();

        Self {
            luminate: luminate.clone(),
            locked: false,
            draft,
            shared,
        }
    }

    fn update(&mut self, message: Message) -> Action<Message> {
        match message {
            Message::InputChanged(value) => {
                self.shared.set(value.clone());
                self.draft = value;
            }
            Message::Unlock => self.locked = false,
        }

        Action::none()
    }

    fn view(&self) -> Element<'_, Message> {
        let luminate = &self.luminate;

        let mut input = Input::new("Type something", &self.draft)
            .label("Draft")
            .hint("Shared with the nested sidebar's inputs page");
        if !self.locked {
            input = input.on_input(Message::InputChanged);
        }

        let unlock: Element<'_, Message> = if self.locked {
            luminate.button(Button::new("Unlock").on_press(Message::Unlock))
        } else {
            Space::new().into()
        };

        column![luminate.input(input), unlock].spacing(15).into()
    }

    /// Runs only when a navigation carried [`Options`].
    fn on_navigate(&mut self, options: Options) {
        self.locked = options.locked;
    }

    /// Runs every time this suspended instance is shown again.
    fn on_resume(&mut self) {
        self.draft = self.shared.get();
    }
}
