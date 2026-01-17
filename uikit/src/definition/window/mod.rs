use iced::Element;

use crate::KitObj;

pub struct UiWindow<'a, Message> {
    pub label: String,
    pub child: Element<'a, Message>,
    pub controls_child: Option<Element<'a, Message>>,

    pub wrapper: bool,
    pub wrapper_close_event: Option<Message>,

    pub kit: &'a KitObj<Message>,
}

impl<'a, Message> UiWindow<'a, Message>
where
    Message: Clone + 'static,
{
    pub fn new(kit: &'a KitObj<Message>, label: String, child: Element<'a, Message>) -> Self {
        UiWindow {
            label,
            child,
            controls_child: None,

            wrapper: false,
            wrapper_close_event: None,

            kit,
        }
    }

    pub fn child(mut self, child: impl Into<Element<'a, Message>>) -> Self {
        self.child = child.into();
        self
    }

    pub fn controls_child(mut self, child: impl Into<Element<'a, Message>>) -> Self {
        self.controls_child = Some(child.into());
        self
    }

    pub fn wrap(mut self, enable: bool) -> Self {
        self.wrapper = enable;
        self
    }

    pub fn close_event(mut self, event: Message) -> Self {
        self.wrapper = true;
        self.wrapper_close_event = Some(event);
        self
    }
}

impl<'a, Message: Clone + 'static> From<UiWindow<'a, Message>> for Element<'a, Message> {
    fn from(value: UiWindow<'a, Message>) -> Self {
        value.kit.constr_window(value)
    }
}
