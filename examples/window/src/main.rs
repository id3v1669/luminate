use iced::{
    Color, Element, Font, Padding, Settings, Task, font, theme,
    widget::{button, column, text},
};
use iced_auravibe::{
    Kit,
    definition::button::props::ButtonHierarchy,
    kit::sonata::{Sonata, components::spring_layer::spring_layer},
    mapper::UIMapper,
};

fn main() -> iced::Result {
    std::panic::set_hook(Box::new(console_error_panic_hook::hook));

    iced::application(move || Data::new(Sonata::new()), Data::update, Data::view)
        .style(|_, _| theme::Style {
            background_color: Color::WHITE,
            text_color: Color::BLACK,
        })
        .run()
}

struct Data {
    uikit: Box<dyn for<'a> Kit<'a, Message>>,
    input_content: String,
}

#[derive(Debug, Clone)]
pub enum Message {
    Pressed,
    InputContentChanged(String),
}

impl Data {
    fn new<K>(kit: K) -> (Self, Task<Message>)
    where
        K: for<'a> Kit<'a, Message> + 'static,
    {
        (
            Self {
                uikit: Box::new(kit),
                input_content: String::new(),
            },
            Task::none(),
        )
    }

    fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::Pressed => println!("Pressed"),
            Message::InputContentChanged(content) => {
                self.input_content = content;
            }
        }

        Task::none()
    }

    fn kit_mapper(&self) -> UIMapper<'_, Message> {
        UIMapper::new(&self.uikit)
    }

    fn view(&self) -> Element<'_, Message> {
        let kit = self.kit_mapper();

        column![spring_layer(
            kit.button("New Button").on_press(Message::Pressed)
        ),]
        .padding(Padding::from(15))
        .spacing(15)
        .into()
    }
}
