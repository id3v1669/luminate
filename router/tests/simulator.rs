//! One end-to-end check through iced's headless renderer: render the
//! current page, click a button in it, and route the resulting message back
//! through `Router::update`.

use iced::Element;
use iced::widget::{button, column, text};
use iced_page_router::{Action, Page, Registry, RouteMessage, Router};
use iced_test::Simulator;

#[derive(Debug, Clone)]
enum ClickerMessage {
    Increment,
}

struct Clicker {
    count: u32,
}

impl Page for Clicker {
    type Message = ClickerMessage;
    type NavigationOptions = ();
    type Context = ();
    type Theme = iced::Theme;
    type Renderer = iced::Renderer;

    fn new((): &(), _: &Registry) -> Self {
        Self { count: 0 }
    }

    fn update(&mut self, message: ClickerMessage) -> Action<ClickerMessage> {
        match message {
            ClickerMessage::Increment => self.count += 1,
        }
        Action::none()
    }

    fn view(&self) -> Element<'_, ClickerMessage> {
        column![
            text(format!("count: {}", self.count)),
            button(text("+")).on_press(ClickerMessage::Increment),
        ]
        .into()
    }
}

#[test]
fn a_click_in_the_rendered_page_reaches_the_page_through_the_router() {
    let mut router: Router<(), iced::Theme, iced::Renderer> = Router::new(Registry::new(), ());
    router.add::<Clicker>("clicker");
    router.navigate::<Clicker>().unwrap();

    let messages: Vec<RouteMessage> = {
        let mut ui = Simulator::new(router.view().expect("a page is current"));
        let _ = ui.find("count: 0").expect("the page rendered");
        let _ = ui.click("+").expect("the button is visible");
        ui.into_messages().collect()
    };
    assert_eq!(messages.len(), 1, "one press, one message");

    for message in messages {
        let _ = router.update(message);
    }
    assert_eq!(router.page::<Clicker>().unwrap().count, 1);

    let mut ui = Simulator::new(router.view().unwrap());
    let _ = ui.find("count: 1").expect("the view reflects the update");
}
