use iced::{
    Alignment, Color, Element, Length, Padding, Renderer, Task, Theme, theme,
    widget::{button, column, row, scrollable, svg, text},
};
use iced_auravibe::{
    Kit,
    definition::button::props::ButtonHierarchy,
    kit::sonata::{
        Sonata,
        components::sidebar::{self, widget::Sidebar},
        utils::{
            spring_layer::spring_layer,
            text::{display::DisplayStyle, text::TextStyle},
        },
    },
    mapper::UIMapper,
};

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

        let content = scrollable(
            column![
                row![
                    kit.button()
                        .label("Toggle")
                        .icon(icon_str)
                        .on_press(Message::Pressed),
                    kit.button()
                        .label("Remove")
                        .hier(ButtonHierarchy::Destructive)
                        .on_press(Message::Pressed),
                ]
                .align_y(Alignment::Center)
                .spacing(10),
                kit.input("Placeholder", &self.input_content)
                    .on_input(Message::InputContentChanged)
                    .label("Label")
                    .hint("Hint for the user")
                    .is_error(self.input_error)
                    .error_msg("Value is wrong"),
                text("Display 2XL")
                    .font(DisplayStyle::Display2xlM.build_font())
                    .size(DisplayStyle::Display2xlM.get_size()),
                text("Display XL")
                    .font(DisplayStyle::DisplayXlM.build_font())
                    .size(DisplayStyle::DisplayXlM.get_size()),
                text("Display LG")
                    .font(DisplayStyle::DisplayLgM.build_font())
                    .size(DisplayStyle::DisplayLgM.get_size()),
                text("Display MD")
                    .font(DisplayStyle::DisplayMdM.build_font())
                    .size(DisplayStyle::DisplayMdM.get_size()),
                text("Display SM")
                    .font(DisplayStyle::DisplaySmM.build_font())
                    .size(DisplayStyle::DisplaySmM.get_size()),
                text("Display XS")
                    .font(DisplayStyle::DisplayXsM.build_font())
                    .size(DisplayStyle::DisplayXsM.get_size()),
                text("Text XL")
                    .font(TextStyle::TextXlM.build_font())
                    .size(TextStyle::TextXlM.get_size()),
                text("Text LG")
                    .font(TextStyle::TextLgM.build_font())
                    .size(TextStyle::TextLgM.get_size()),
                text("Text MD")
                    .font(TextStyle::TextMdM.build_font())
                    .size(TextStyle::TextMdM.get_size()),
                text("Text SM")
                    .font(TextStyle::TextSmM.build_font())
                    .size(TextStyle::TextSmM.get_size()),
                text("Text XS")
                    .font(TextStyle::TextXsM.build_font())
                    .size(TextStyle::TextXsM.get_size()),
                spring_layer(kit.button().label("Action").on_press(Message::Pressed))
            ]
            .height(Length::Fill)
            .width(Length::Fill)
            .padding(Padding::from(15))
            .spacing(15),
        );

        row![
            kit.sidebar(vec![
                kit.button()
                    .label("Home")
                    .hier(ButtonHierarchy::Tertiary)
                    .width(Length::Fill)
                    .on_press(Message::Pressed)
                    .into(),
                kit.button()
                    .label("Dashboard")
                    .hier(ButtonHierarchy::Secondary)
                    .width(Length::Fill)
                    .on_press(Message::Pressed)
                    .into(),
                kit.button()
                    .label("Messages")
                    .hier(ButtonHierarchy::Tertiary)
                    .width(Length::Fill)
                    .on_press(Message::Pressed)
                    .into(),
                kit.button()
                    .label("Settings")
                    .hier(ButtonHierarchy::Tertiary)
                    .width(Length::Fill)
                    .on_press(Message::Pressed)
                    .into(),
            ])
            .width(200),
            content
        ]
        .into()
    }
}
