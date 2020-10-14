// #![deny(missing_docs)]  // annoying while developing
#![deny(missing_debug_implementations)]
#![deny(unused_results)]
#![forbid(unsafe_code)]
#![forbid(rust_2018_idioms)]

mod conversion;
mod handler;
mod application;
mod element;

pub mod settings;
pub mod widget;
pub mod executor;
pub mod keyboard;
pub mod mouse;
pub mod window;

pub use application::Application;
pub use handler::Handler;
pub use settings::Settings;
pub use element::Element;
pub use executor::Executor;

#[cfg(feature = "wgpu")]
pub use iced_wgpu as renderer;

#[doc(no_inline)]
pub use iced_native::{
    Align, Background, Color, Font, HorizontalAlignment, Length, Point,
    Rectangle, Size, Vector, VerticalAlignment, futures, Command, Subscription,
};

#[doc(no_inline)]
pub use widget::*;

#[cfg(all(
    any(feature = "tokio", feature = "async-std"),
))]
#[cfg_attr(docsrs, doc(cfg(any(feature = "tokio", feature = "async-std"))))]
pub mod time;

#[doc(no_inline)]
pub use baseview;