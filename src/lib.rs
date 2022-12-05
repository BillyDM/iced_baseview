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
#[cfg(feature = "wgpu")]
mod wrapper;

pub mod clipboard;
pub mod executor;
pub mod settings;
pub mod time;
pub mod widget;

pub mod baseview {
    #[cfg(feature = "glow")]
    pub use baseview::gl;
    pub use baseview::{Size, WindowOpenOptions, WindowScalePolicy};
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
        type Renderer = iced_wgpu::Renderer;
        type Compositor<Theme> = iced_wgpu::window::Compositor<Theme>;
        pub use iced_wgpu as backend;
    } else {
        type Renderer = iced_glow::Renderer;
        type Compositor<Theme> = iced_glow::window::Compositor<Theme>;
        pub use iced_glow as backend;
    }
}

pub use iced_native::Theme as DefaultTheme;

#[doc(no_inline)]
pub use widget::*;
