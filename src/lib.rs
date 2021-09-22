// #![deny(missing_docs)]  // annoying while developing
#![deny(missing_debug_implementations)]
#![deny(unused_results)]
//#![forbid(unsafe_code)]
#![forbid(rust_2018_idioms)]

mod application;
mod conversion;
mod element;
mod proxy;
mod window;

pub mod executor;
pub mod keyboard;
pub mod mouse;
pub mod settings;
pub mod widget;

pub use application::Application;
pub use element::Element;
pub use executor::Executor;
pub use settings::Settings;
pub use window::{IcedWindow, WindowHandle, WindowSubs};

#[cfg(feature = "wgpu")]
type Renderer = iced_wgpu::Renderer;
#[cfg(feature = "wgpu")]
type Compositor = iced_wgpu::window::Compositor;
#[cfg(feature = "wgpu")]
pub use iced_wgpu as renderer;

#[cfg(feature = "glow")]
#[cfg(not(feature = "wgpu"))]
type Renderer = iced_glow::Renderer;
#[cfg(feature = "glow")]
#[cfg(not(feature = "wgpu"))]
type Compositor = iced_glow::window::Compositor;
#[cfg(feature = "glow")]
#[cfg(not(feature = "wgpu"))]
pub use iced_glow as renderer;

#[doc(no_inline)]
pub use iced_native::{
    futures, Align, Background, Color, Command, Font, HorizontalAlignment,
    Length, Point, Rectangle, Size, Subscription, Vector, VerticalAlignment,
};

#[doc(no_inline)]
pub use widget::*;

#[cfg(all(any(feature = "tokio", feature = "async-std"),))]
#[cfg_attr(docsrs, doc(cfg(any(feature = "tokio", feature = "async-std"))))]
pub mod time;
