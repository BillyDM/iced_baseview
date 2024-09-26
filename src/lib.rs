//! A windowing shell for Iced, on top of [`baseview`].
//!
//! Largely stolen from (MIT licensed) [`iced_winit`].
//!
//! [`baseview`]: https://github.com/RustAudio/baseview
//! [`iced_winit`]: https://github.com/iced-rs/iced/tree/master/winit
#![deny(
    // missing_debug_implementations,
    // missing_docs,
    unused_results,
    clippy::extra_unused_lifetimes,
    clippy::from_over_into,
    clippy::needless_borrow,
    clippy::new_without_default,
    clippy::useless_conversion,
)]
#![forbid(rust_2018_idioms)]
#![allow(clippy::inherent_to_string, clippy::type_complexity)]
#![cfg_attr(docsrs, feature(doc_auto_cfg))]
pub use iced_graphics as graphics;
pub use iced_graphics::Viewport;
use iced_renderer::Compositor;
pub use iced_runtime as runtime;
pub use iced_runtime::core::{
    self, alignment, border, color, gradient, padding, theme, Alignment, Background, Border, Color,
    ContentFit, Degrees, Gradient, Length, Padding, Pixels, Point, Radians, Rectangle, Rotation,
    Shadow, Size, Theme, Transformation, Vector,
};
pub use iced_runtime::futures;

pub use alignment::Horizontal::{Left, Right};
pub use alignment::Vertical::{Bottom, Top};
pub use Alignment::Center;
pub use Length::{Fill, FillPortion, Shrink};

pub mod task {
    //! Create runtime tasks.
    pub use crate::runtime::task::{Handle, Task};
}

pub mod application;
pub mod clipboard;
pub mod conversion;
pub mod settings;
pub mod window;

#[cfg(feature = "system")]
pub mod system;

mod error;
mod position;
mod proxy;

pub use application::{Appearance, Application, DefaultStyle};
pub use clipboard::Clipboard;
pub use error::Error;
pub use event::Event;
pub use executor::Executor;
pub use font::Font;
pub use position::Position;
#[cfg(feature = "trace")]
pub use program::Profiler;
pub use proxy::Proxy;
pub use renderer::Renderer;
pub use settings::{GraphicsSettings, IcedBaseviewSettings, Settings};
pub use task::Task;
pub use window::WindowSubs;

pub mod baseview {
    pub use baseview::{Size, WindowOpenOptions, WindowScalePolicy};
}

pub use iced_widget::renderer;

pub mod executor {
    //! Choose your preferred executor to power your application.
    pub use iced_runtime::futures::Executor;

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
    pub type Default = iced_runtime::futures::backend::default::Executor;
}

pub mod font {
    //! Load and use fonts.
    pub use crate::core::font::*;
    pub use crate::runtime::font::*;
}

pub mod event {
    //! Handle events of a user interface.
    pub use crate::core::event::{Event, Status};
    pub use iced_runtime::futures::event::{listen, listen_raw, listen_with};
}

pub mod keyboard {
    //! Listen and react to keyboard events.
    pub use crate::core::keyboard::key;
    pub use crate::core::keyboard::{Event, Key, Location, Modifiers};
    pub use iced_runtime::futures::keyboard::{on_key_press, on_key_release};
}

pub mod mouse {
    //! Listen and react to mouse events.
    pub use crate::core::mouse::{Button, Cursor, Event, Interaction, ScrollDelta};
}

pub mod overlay {
    //! Display interactive elements on top of other widgets.

    /// A generic overlay.
    ///
    /// This is an alias of an [`overlay::Element`] with a default `Renderer`.
    ///
    /// [`overlay::Element`]: crate::core::overlay::Element
    pub type Element<'a, Message, Theme = crate::Renderer, Renderer = crate::Renderer> =
        crate::core::overlay::Element<'a, Message, Theme, Renderer>;

    pub use iced_widget::overlay::*;
}

pub mod touch {
    //! Listen and react to touch events.
    pub use crate::core::touch::{Event, Finger};
}

#[allow(hidden_glob_reexports)]
pub mod widget {
    //! Use the built-in widgets or create your own.
    pub use iced_widget::*;

    // We hide the re-exported modules by `iced_widget`
    mod core {}
    mod graphics {}
    mod native {}
    mod renderer {}
    mod style {}
    mod runtime {}
}

/// A generic widget.
///
/// This is an alias of an `iced_native` element with a default `Renderer`.
pub type Element<'a, Message, Theme = crate::Theme, Renderer = crate::Renderer> =
    crate::core::Element<'a, Message, Theme, Renderer>;

/// The result of running an iced program.
pub type Result = std::result::Result<(), Error>;

/// Runs the [`Application`] in a child window.
pub fn open_parented<A, W>(
    parent: &W,
    flags: A::Flags,
    settings: Settings,
) -> window::WindowHandle<A::Message>
where
    A: Application + Send + 'static,
    A::Flags: Send,
    W: raw_window_handle::HasRawWindowHandle,
{
    window::IcedWindow::<A>::open_parented::<W, Compositor>(parent, flags, settings)
}

pub fn open_blocking<A>(flags: A::Flags, settings: Settings)
where
    A: Application + Send + 'static,
    A::Flags: Send,
{
    window::IcedWindow::<A>::open_blocking::<Compositor>(flags, settings)
}
