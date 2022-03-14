// #![deny(missing_docs)]  // annoying while developing
#![deny(missing_debug_implementations)]
#![deny(unused_results)]
//#![forbid(unsafe_code)]
#![forbid(rust_2018_idioms)]

#[doc(no_inline)]
pub use iced_native::*;

mod application;
pub mod conversion;
mod element;
mod proxy;
mod window;

pub mod clipboard;
pub mod executor;
pub mod settings;
pub mod time;
pub mod widget;

pub use application::Application;
pub use element::Element;
pub use executor::Executor;
pub use settings::{IcedBaseviewSettings, Settings};
pub use window::{IcedWindow, WindowHandle, WindowQueue, WindowSubs};

#[cfg(all(feature = "wgpu", feature = "glow"))]
compile_error!("Can't use both 'wgpu' and 'glow' features");

#[cfg(feature = "wgpu")]
type Renderer = iced_wgpu::Renderer;
#[cfg(feature = "wgpu")]
type Compositor = iced_wgpu::window::Compositor;
#[cfg(feature = "wgpu")]
pub use iced_wgpu as backend;

#[cfg(feature = "glow")]
#[cfg(not(feature = "wgpu"))]
type Renderer = iced_glow::Renderer;
#[cfg(feature = "glow")]
#[cfg(not(feature = "wgpu"))]
type Compositor = iced_glow::window::Compositor;
#[cfg(feature = "glow")]
#[cfg(not(feature = "wgpu"))]
pub use iced_glow as backend;

#[doc(no_inline)]
pub use widget::*;
