pub mod builder;
mod vars;
pub mod widget;

use iced::Element;
use widget::Sidebar;

#[macro_export]
macro_rules! sidebar {
    () => (
        $Sidebar::new()
    );
    ($($x:expr),+ $(,)?) => (
        $Sidebar::with_children([$($Element::from($x)),+])
    );
}
