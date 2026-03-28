use iced::Element;

use crate::KitObj;

pub mod props;

pub struct UiInput<'a, Message> {
    pub value: &'a str,
    pub placeholder: &'a str,
    pub label: Option<&'a str>,
    pub hint: Option<&'a str>,
    pub is_error: Option<bool>,
    pub error_msg: Option<&'a str>,

    pub on_input: Option<Box<dyn Fn(String) -> Message + 'static>>,
    pub kit: &'a KitObj<Message>,
}

impl<'a, Message> UiInput<'a, Message>
where
    Message: Clone + 'static,
{
    pub fn new(kit: &'a KitObj<Message>, placeholder: &'a str, value: &'a str) -> Self {
        UiInput {
            value,
            placeholder,
            label: None,
            hint: None,
            is_error: None,
            error_msg: None,

            on_input: None,
            kit,
        }
    }

    pub fn on_input(mut self, on_input: impl Fn(String) -> Message + 'static) -> Self {
        self.on_input = Some(Box::new(on_input));
        self
    }

    pub fn label(mut self, label: &'a str) -> Self {
        self.label = Some(label);
        self
    }

    pub fn hint(mut self, hint: &'a str) -> Self {
        self.hint = Some(hint);
        self
    }

    pub fn is_error(mut self, state: bool) -> Self {
        self.is_error = Some(state);
        self
    }

    pub fn error_msg(mut self, msg: &'a str) -> Self {
        self.error_msg = Some(msg);
        self
    }
}

impl<'a, Message> From<UiInput<'a, Message>> for Element<'a, Message>
where
    Message: Clone + 'static,
{
    fn from(value: UiInput<'a, Message>) -> Self {
        value.kit.constr_input(value)
    }
}
