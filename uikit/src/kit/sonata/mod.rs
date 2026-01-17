use std::marker::PhantomData;

use iced::Element;

use crate::Kit;
use crate::definition::button::UiButton;

pub mod button;
// pub mod focusabe_container;
pub mod input;
pub mod text;
pub mod window;

#[derive(Clone)]
pub struct Sonata<Message> {
    _marker: PhantomData<Message>,
}

impl<Message: Clone + 'static> Sonata<Message> {
    pub fn new() -> Self {
        Self {
            _marker: PhantomData,
        }
    }
}

impl<Message: Clone + 'static> Default for Sonata<Message> {
    fn default() -> Self {
        Self {
            _marker: PhantomData,
        }
    }
}

impl<'a, Message: Clone + 'static> Kit<'a, Message> for Sonata<Message> {
    fn constr_button(&self, params: UiButton<Message>) -> Element<'static, Message> {
        Self::button(&self, params)
    }

    fn constr_input(
        &self,
        input: crate::definition::input::UiInput<Message>,
    ) -> Element<'static, Message> {
        Self::input(&self, input)
    }

    fn constr_window(
        &self,
        window: crate::definition::window::UiWindow<'a, Message>,
    ) -> Element<'a, Message> {
        self.window(window)
    }
}
