// #![deny(missing_docs)]  // annoying while developing
#![deny(missing_debug_implementations)]
#![deny(unused_results)]
//#![forbid(unsafe_code)]
#![forbid(rust_2018_idioms)]

use cfg_if::cfg_if;
#[doc(no_inline)]
pub use iced_native::*;

mod application;
pub mod conversion;
mod element;
mod proxy;
mod window;
mod wrapper;

pub mod clipboard;
pub mod executor;
pub mod settings;
pub mod time;
pub mod widget;

pub mod baseview {
    pub use baseview::{Size, WindowOpenOptions, WindowScalePolicy};
    #[cfg(feature = "glow")]
    pub use baseview::gl;
}

pub use application::Application;
pub use element::Element;
pub use executor::Executor;
pub use settings::{IcedBaseviewSettings, Settings};
pub use window::{IcedWindow, WindowHandle, WindowQueue, WindowSubs};

#[cfg(all(feature = "wgpu", feature = "glow"))]
compile_error!("Can't use both 'wgpu' and 'glow' features");

cfg_if! {
    if #[cfg(feature = "wgpu")] {
        pub type Theme = iced_wgpu::Theme;
        type Renderer = iced_wgpu::Renderer;
        type Compositor = iced_wgpu::window::Compositor<Theme>;
        pub use iced_wgpu as backend;
    } else {
        pub type Theme = iced_glow::Theme;
        type Renderer = iced_glow::Renderer;
        type Compositor = iced_glow::window::Compositor<Theme>;
        pub use iced_glow as backend;
    }
}

#[doc(no_inline)]
pub use widget::*;
