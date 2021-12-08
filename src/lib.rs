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
pub use window::{IcedWindow, WindowHandle, WindowQueue, WindowSubs};

#[cfg(all(feature = "with-wgpu", feature = "with-glow"))]
compile_error!("Can't use both 'with-wgpu' and 'with-glow' features");

#[cfg(feature = "with-wgpu")]
type Renderer = iced_wgpu::Renderer;
#[cfg(feature = "with-wgpu")]
type Compositor = iced_wgpu::window::Compositor;
#[cfg(feature = "with-wgpu")]
pub use iced_wgpu as renderer;

#[cfg(feature = "with-glow")]
#[cfg(not(feature = "with-wgpu"))]
type Renderer = iced_glow::Renderer;
#[cfg(feature = "with-glow")]
#[cfg(not(feature = "with-wgpu"))]
type Compositor = iced_glow::window::Compositor;
#[cfg(feature = "with-glow")]
#[cfg(not(feature = "with-wgpu"))]
pub use iced_glow as renderer;

#[doc(no_inline)]
pub use iced_native::{
    alignment::Alignment, alignment::Horizontal, alignment::Vertical, futures,
    Background, Color, Command, Font, Length, Point, Rectangle, Size,
    Subscription, Vector,
};

#[doc(no_inline)]
pub use widget::*;

#[cfg(all(any(feature = "with-tokio", feature = "with-async-std"),))]
#[cfg_attr(docsrs, doc(cfg(any(feature = "with-tokio", feature = "with-async-std"))))]
pub mod time;
