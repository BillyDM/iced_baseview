// #![deny(missing_docs)]  // annoying while developing
#![deny(missing_debug_implementations)]
#![deny(unused_results)]
//#![forbid(unsafe_code)]
#![forbid(rust_2018_idioms)]

#[cfg(all(feature = "wgpu", feature = "glow"))]
compile_error!("Can't use both 'wgpu' and 'glow' features");

#[cfg(not(any(feature = "wgpu", feature = "glow")))]
compile_error!("Must use either 'wgpu' or 'glow' feature");

use cfg_if::cfg_if;
#[doc(no_inline)]
pub use iced_native::*;

mod application;
pub mod conversion;
mod proxy;
mod window;
#[cfg(feature = "wgpu")]
mod wrapper;

pub mod clipboard;
pub mod settings;
pub mod widget;

pub mod baseview {
    #[cfg(feature = "glow")]
    pub use baseview::gl;
    pub use baseview::{Size, WindowOpenOptions, WindowScalePolicy};
}

pub mod executor {
    //! Choose your preferred executor to power your application.
    pub use iced_native::Executor;

    /// A default cross-platform executor.
    ///
    /// - On native platforms, it will use:
    ///   - `iced_futures::backend::native::tokio` when the `tokio` feature is enabled.
    ///   - `iced_futures::backend::native::async-std` when the `async-std` feature is
    ///     enabled.
    ///   - `iced_futures::backend::native::smol` when the `smol` feature is enabled.
    ///   - `iced_futures::backend::native::thread_pool` otherwise.
    ///
    /// - On Wasm, it will use `iced_futures::backend::wasm::wasm_bindgen`.
    pub type Default = iced_futures::backend::default::Executor;
}

pub mod time {
    //! Listen and react to time.
    pub use iced_core::time::{Duration, Instant};

    pub use iced_futures::backend::default::time::*;
}

pub mod theme {
    //! Default theme

    pub use iced_native::Theme;
}

pub use application::Application;
pub use executor::Executor;
pub use settings::{IcedBaseviewSettings, Settings};
pub use window::{IcedWindow, WindowHandle, WindowQueue, WindowSubs};

#[doc(no_inline)]
pub use widget::*;

cfg_if! {
    if #[cfg(feature = "wgpu")] {
        pub use iced_wgpu as backend;
    } else {
        pub use iced_glow as backend;
    }
}

type Renderer = backend::Renderer;
type Compositor<Theme> = backend::window::Compositor<Theme>;

/// A generic widget.
///
/// This is an alias of an `iced_native` element with a default `Renderer`.
pub type Element<'a, Message> =
    iced_native::Element<'a, Message, crate::backend::Renderer>;
