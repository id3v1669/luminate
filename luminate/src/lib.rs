#![doc = include_str!("../README.md")]

pub mod descriptor;
pub mod theme;
pub mod widget;

mod luminate;

pub use iced;
pub use iced_animate as animate;
pub use iced_page_router as router;
pub use iced_texture_cache as texture;
pub use luminate::Luminate;
pub use theme::Theme;

/// The renderer every Luminate element is drawn by: `iced_texture_cache`'s,
/// which iced's `application(..)` picks up from the view's element type.
pub type Renderer = iced_texture_cache::Renderer;

/// An iced element drawn with [`Theme`] by [`Renderer`]: what every
/// [`Luminate`] builder returns and what a page's `view` produces.
pub type Element<'a, Message> = iced::Element<'a, Message, Theme, Renderer>;

/// A router whose pages draw with [`Theme`] and receive a [`Luminate`] as
/// their context (the default). Pages write
/// `type Context = Luminate; type Theme = iced_luminate::Theme; type Renderer = iced_luminate::Renderer;`.
pub type Router<Context = Luminate> = iced_page_router::Router<Context, Theme, Renderer>;
