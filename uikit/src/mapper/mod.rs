use std::marker::PhantomData;

use iced::widget::button;

use crate::{
    KitObj,
    definition::{button::UiButton, input::UiInput, window::UiWindow},
};

// pub mod button;

// pub type MessageMapper<AppMsg> = Arc<dyn Fn(KitMessage) -> AppMsg + Send + Sync + 'static>;

pub struct UIMapper<'a, Message> {
    kit: &'a KitObj<Message>,
    _marker: PhantomData<Message>,
}

impl<'a, Message> UIMapper<'a, Message>
where
    Message: Clone + 'static,
{
    pub fn new(kit: &'a KitObj<Message>) -> Self {
        Self {
            kit,
            _marker: PhantomData,
        }
    }

    pub fn button(&self, label: impl Into<String>, on_press: Message) -> UiButton<'a, Message> {
        UiButton::new(&self.kit, label, on_press)
    }

    pub fn input(&self, placeholder: &'a str, value: &'a str) -> UiInput<'a, Message> {
        UiInput::new(&self.kit, placeholder, value)
    }

    pub fn window(&self, label: impl Into<String>) -> UiWindow<'a, Message> {
        UiWindow::new(&self.kit, label.into(), button("asdasd").into())
    }

    // pub fn build(&self, btn: UiButton<Message>) -> Element<'static, Message> {
    //     self.kit.constr_button(btn)
    // }

    // pub fn raw(&self, el: Element<'static, KitMessage>) -> Element<'a, AppMsg> {
    //     let mapper = self.get_mapper();
    //     el.map(move |kmsg| (mapper)(kmsg))
    // }
}
