//! A [`baseview`] integration for [`iced`]
//!
//! [`iced`]: https://github.com/iced-rs/iced/
//! [`baseview`]: https://github.com/RustAudio/baseview/
#![deny(
    unused_results,
    clippy::extra_unused_lifetimes,
    clippy::from_over_into,
    clippy::needless_borrow,
    clippy::new_without_default,
    clippy::useless_conversion
)]
#![forbid(rust_2018_idioms)]
#![allow(clippy::inherent_to_string, clippy::type_complexity)]
#![cfg_attr(docsrs, feature(doc_cfg))]

#[cfg(all(feature = "wgpu", feature = "glow"))]
compile_error!("Can't use both 'wgpu' and 'glow' features");

#[cfg(not(any(feature = "wgpu", feature = "glow")))]
compile_error!("Must use either 'wgpu' or 'glow' feature");

pub mod clipboard;
pub mod settings;
pub mod widget;
pub mod window;

mod application;
mod conversion;
mod error;
mod proxy;
mod wrapper;

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

#[doc(no_inline)]
pub use iced_native::{
    alignment, command, event, keyboard, mouse, overlay, subscription, Alignment, Background,
    Color, Command, ContentFit, Debug, Event, Font, Hasher, Layout, Length, Overlay, Padding,
    Point, Rectangle, Size, Subscription, Vector,
};

cfg_if::cfg_if! {
    if #[cfg(feature = "wgpu")] {
        pub use iced_wgpu as renderer;
        use iced_graphics::window::Compositor as IGCompositor;
    } else {
        pub use iced_glow as renderer;
        use iced_graphics::window::GLCompositor as IGCompositor;
    }
}

pub use renderer::Renderer;
pub use settings::Settings;

use raw_window_handle::HasRawWindowHandle;
use window::{IcedWindow, WindowQueue, WindowSubs};

/// A generic widget.
///
/// This is an alias of an `iced_native` element with a default `Renderer`.
pub type Element<'a, Message, Theme> =
    iced_native::Element<'a, Message, crate::renderer::Renderer<Theme>>;

pub trait Application: Sized + Send {
    /// The [`Executor`] that will run commands and subscriptions.
    ///
    /// The [default executor] can be a good starting point!
    ///
    /// [`Executor`]: Self::Executor
    /// [default executor]: crate::executor::Default
    type Executor: iced_native::Executor;

    /// The type of __messages__ your [`Application`] will produce.
    type Message: std::fmt::Debug + Send;

    /// The theme of your [`Application`].
    type Theme: Default + iced_native::application::StyleSheet;

    /// The data needed to initialize your [`Application`].
    type Flags: Send;

    /// Initializes the [`Application`] with the flags provided to
    /// [`run`] as part of the [`Settings`].
    ///
    /// Here is where you should return the initial state of your app.
    ///
    /// Additionally, you can return a [`Command`] if you need to perform some
    /// async action in the background on startup. This is useful if you want to
    /// load state from a file, perform an initial HTTP request, etc.
    ///
    /// [`run`]: Self::run
    fn new(flags: Self::Flags) -> (Self, iced_native::Command<Self::Message>);

    /// Returns the current title of the [`Application`].
    ///
    /// This title can be dynamic! The runtime will automatically update the
    /// title of your application when necessary.
    fn title(&self) -> String;

    /// Handles a __message__ and updates the state of the [`Application`].
    ///
    /// This is where you define your __update logic__. All the __messages__,
    /// produced by either user interactions or commands, will be handled by
    /// this method.
    ///
    /// Any [`Command`] returned will be executed immediately in the background.
    fn update(
        &mut self,
        window: &mut WindowQueue,
        message: Self::Message,
    ) -> iced_native::Command<Self::Message>;

    /// Returns the widgets to display in the [`Application`].
    ///
    /// These widgets can produce __messages__ based on user interaction.
    fn view(&self) -> Element<'_, Self::Message, Self::Theme>;

    /// Returns the current [`Theme`] of the [`Application`].
    ///
    /// [`Theme`]: Self::Theme
    fn theme(&self) -> Self::Theme {
        Self::Theme::default()
    }

    /// Returns the current `Style` of the [`Theme`].
    ///
    /// [`Theme`]: Self::Theme
    fn style(&self) -> <Self::Theme as iced_native::application::StyleSheet>::Style {
        <Self::Theme as iced_native::application::StyleSheet>::Style::default()
    }

    /// Returns the event [`Subscription`] for the current state of the
    /// application.
    ///
    /// A [`Subscription`] will be kept alive as long as you keep returning it,
    /// and the __messages__ produced will be handled by
    /// [`update`](#tymethod.update).
    ///
    /// By default, this method returns an empty [`Subscription`].
    fn subscription(
        &self,
        _window_subs: &mut WindowSubs<Self::Message>,
    ) -> iced_native::Subscription<Self::Message> {
        iced_native::Subscription::none()
    }

    /// Returns the [`WindowScalePolicy`] that the [`Application`] should use.
    ///
    /// By default, it returns `WindowScalePolicy::SystemScaleFactor`.
    ///
    /// [`WindowScalePolicy`]: ../settings/enum.WindowScalePolicy.html
    /// [`Application`]: trait.Application.html
    fn scale_policy(&self) -> ::baseview::WindowScalePolicy {
        ::baseview::WindowScalePolicy::SystemScaleFactor
    }

    /// Returns whether the [`Application`] should be terminated.
    ///
    /// By default, it returns `false`.
    fn should_exit(&self) -> bool {
        false
    }
    fn renderer_settings() -> crate::renderer::Settings {
        Default::default()
    }
}

struct Instance<A: Application>(A);

impl<A> application::Application for Instance<A>
where
    A: Application,
{
    type Flags = A::Flags;
    type Renderer = renderer::Renderer<A::Theme>;
    type Message = A::Message;

    fn new(flags: Self::Flags) -> (Self, iced_native::Command<A::Message>) {
        let (app, command) = A::new(flags);

        (Instance(app), command)
    }

    fn title(&self) -> String {
        self.0.title()
    }

    fn theme(&self) -> A::Theme {
        self.0.theme()
    }

    fn style(&self) -> <A::Theme as iced_native::application::StyleSheet>::Style {
        self.0.style()
    }
    fn update(
        &mut self,
        window: &mut WindowQueue,
        message: Self::Message,
    ) -> iced_native::Command<Self::Message> {
        self.0.update(window, message)
    }

    fn view(&self) -> iced_native::Element<'_, Self::Message, Self::Renderer> {
        self.0.view()
    }

    fn subscription(
        &self,
        window_subs: &mut WindowSubs<A::Message>,
    ) -> iced_native::Subscription<Self::Message> {
        self.0.subscription(window_subs)
    }

    fn scale_policy(&self) -> ::baseview::WindowScalePolicy {
        self.0.scale_policy()
    }

    fn should_exit(&self) -> bool {
        self.0.should_exit()
    }

    fn renderer_settings() -> crate::renderer::Settings {
        A::renderer_settings()
    }
}

/// Open a new window that blocks the current thread until the window is destroyed.
///
/// * `settings` - The settings of the window.
pub fn open_blocking<A: Application>(settings: Settings<A::Flags>)
where
    A: 'static,
{
    IcedWindow::<Instance<A>>::open_blocking::<A::Executor, renderer::window::Compositor<A::Theme>>(
        settings,
    );
}

/// Open a new child window.
///
/// * `parent` - The parent window.
/// * `settings` - The settings of the window.
pub fn open_parented<A: Application, P: HasRawWindowHandle>(
    parent: &P,
    settings: Settings<A::Flags>,
) -> window::WindowHandle<A::Message>
where
    A: 'static,
{
    IcedWindow::<Instance<A>>::open_parented::<A::Executor, renderer::window::Compositor<A::Theme>, P>(
        parent, settings,
    )
}

/// Open a new window as if it had a parent window.
///
/// * `settings` - The settings of the window.
pub fn open_as_if_parented<A: Application>(
    settings: Settings<A::Flags>,
) -> window::WindowHandle<A::Message>
where
    A: 'static,
{
    IcedWindow::<Instance<A>>::open_as_if_parented::<
        A::Executor,
        renderer::window::Compositor<A::Theme>,
    >(settings)
}
