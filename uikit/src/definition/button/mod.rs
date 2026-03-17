use iced::{Element, Length};

use crate::{
    KitObj,
    definition::button::props::{ButtonHierarchy, ButtonSize, UiButtonProperties},
};

pub mod props;

pub struct UiButton<'a, Message> {
    pub label: String,
    pub props: UiButtonProperties,
    pub width: Length,

    pub on_press: Option<Message>,
    pub kit: &'a KitObj<Message>,
}

impl<'a, Message> UiButton<'a, Message>
where
    Message: Clone + 'static,
{
    pub fn new(kit: &'a KitObj<Message>, label: impl Into<String>) -> Self {
        UiButton {
            label: label.into(),
            on_press: None,
            props: UiButtonProperties::default(),
            width: Length::Shrink,
            kit,
        }
    }

    pub fn on_press(mut self, event: Message) -> Self {
        self.on_press = Some(event);
        self
    }

    pub fn width(mut self, width: Length) -> Self {
        self.width = width;
        self
    }

    pub fn size(mut self, size: ButtonSize) -> Self {
        self.props.set_size(size);
        self
    }

    pub fn hier(mut self, hier: ButtonHierarchy) -> Self {
        self.props.set_hier(hier);
        self
    }
}

impl<'a, Message: Clone + 'static> From<UiButton<'a, Message>> for Element<'a, Message> {
    fn from(value: UiButton<'a, Message>) -> Self {
        value.kit.constr_button(value)
    }
}
