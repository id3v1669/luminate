use iced::{Element, Length};

use crate::KitObj;

pub struct UiSidebar<'a, Message> {
    pub content: Vec<Element<'a, Message>>,
    pub width: Length,
    pub expand: bool,

    pub kit: &'a KitObj<Message>,
}

impl<'a, Message> UiSidebar<'a, Message>
where
    Message: Clone + 'static,
{
    pub fn new(kit: &'a KitObj<Message>, children: Vec<Element<'a, Message>>) -> Self {
        UiSidebar {
            content: children,
            width: Length::Fill,
            expand: true,

            kit,
        }
    }

    pub fn width(mut self, width: impl Into<Length>) -> Self {
        self.width = width.into();
        self
    }
}

impl<'a, Message: Clone + 'static> From<UiSidebar<'a, Message>> for Element<'a, Message> {
    fn from(value: UiSidebar<'a, Message>) -> Self {
        value.kit.constr_sidebar(value)
    }
}
