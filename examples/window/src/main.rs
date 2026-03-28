use iced::{
    Color, Element, Length, Padding, Task, theme,
    widget::{column, svg},
};
use iced_auravibe::{Kit, kit::sonata::Sonata, mapper::UIMapper};

fn main() -> iced::Result {
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
    input_error: bool,
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
                input_error: false,
            },
            Task::none(),
        )
    }

    fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::Pressed => self.input_error = !self.input_error,
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
        let icon_str = concat!(env!("CARGO_MANIFEST_DIR"), "/resources/plus.svg");

        column![
            svg(icon_str),
            kit.button()
                .label("Toggle")
                .icon(icon_str)
                .on_press(Message::Pressed),
            kit.input("Placeholder", &self.input_content)
                .on_input(Message::InputContentChanged)
                .hint("Hint for the user")
                .is_error(self.input_error)
                .error_msg("Value is wrong"),
        ]
        .height(Length::Fill)
        .width(Length::Fill)
        .padding(Padding::from(15))
        .spacing(15)
        .into()
    }
}
