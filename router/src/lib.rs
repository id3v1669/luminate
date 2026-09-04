#![doc = include_str!("../README.md")]

mod action;
mod erased;
mod error;
mod history;
mod message;
mod page;
mod registry;
mod router;

pub use action::Action;
pub use error::NavigationError;
pub use message::{Navigation, PageMessage, Payload, RouteMessage};
pub use page::{Lifecycle, Page};
pub use registry::{Key, Registry, Shared};
pub use router::{PageInfo, Router};
