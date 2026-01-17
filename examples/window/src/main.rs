use iced::{Color, Element, Task, theme};
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
}

#[derive(Debug, Clone)]
pub enum Message {}

impl Data {
    fn new<K>(kit: K) -> (Self, Task<Message>)
    where
        K: for<'a> Kit<'a, Message> + 'static,
    {
        (
            Self {
                uikit: Box::new(kit),
            },
            Task::none(),
        )
    }

    fn update(&mut self, _: Message) -> Task<Message> {
        Task::none()
    }

    fn kit_mapper(&self) -> UIMapper<'_, Message> {
        UIMapper::new(&self.uikit)
    }

    fn view(&self) -> Element<'_, Message> {
        let kit = self.kit_mapper();

        kit.window("New Window").into()
    }
}
