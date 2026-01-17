use iced::Element;

use crate::definition::{button::UiButton, input::UiInput, window::UiWindow};

pub mod definition;
pub mod kit;
pub mod mapper;

pub type KitObj<Message> = Box<dyn for<'a> Kit<'a, Message> + 'static>;

pub trait Kit<'a, Message: Clone + 'static> {
    // Primitive
    fn constr_button(&self, btn: UiButton<'a, Message>) -> Element<'a, Message>;
    fn constr_input(&self, input: UiInput<'a, Message>) -> Element<'a, Message>;

    // Complicated
    fn constr_window(&self, window: UiWindow<'a, Message>) -> Element<'a, Message>;
}
