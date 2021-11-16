use crate::{
    Color, Command, Element, Executor, Subscription, WindowQueue, WindowSubs,
};

use baseview::WindowScalePolicy;

mod state;

pub use state::State;

/// An interactive cross-platform application.
///
///
/// TODO: Update this example to the correct `iced_baseview` use case.
///
///
/// This trait is the main entrypoint of Iced. Once implemented, you can run
/// your GUI application by simply calling [`run`](#method.run).
///
/// - On native platforms, it will run in its own window.
/// - On the web, it will take control of the `<title>` and the `<body>` of the
///   document.
///
/// An [`Application`] can execute asynchronous actions by returning a
/// [`Command`](struct.Command.html) in some of its methods. If
/// you do not intend to perform any background work in your program, the
/// [`Sandbox`](trait.Sandbox.html) trait offers a simplified interface.
///
/// When using an [`Application`] with the `debug` feature enabled, a debug view
/// can be toggled by pressing `F12`.
///
/// [`Application`]: trait.Application.html
///
/// # Examples
/// [The repository has a bunch of examples] that use the [`Application`] trait:
///
/// - [`clock`], an application that uses the [`Canvas`] widget to draw a clock
/// and its hands to display the current time.
/// - [`download_progress`], a basic application that asynchronously downloads
/// a dummy file of 100 MB and tracks the download progress.
/// - [`events`], a log of native events displayed using a conditional
/// [`Subscription`].
/// - [`pokedex`], an application that displays a random Pokédex entry (sprite
/// included!) by using the [PokéAPI].
/// - [`solar_system`], an animated solar system drawn using the [`Canvas`] widget
/// and showcasing how to compose different transforms.
/// - [`stopwatch`], a watch with start/stop and reset buttons showcasing how
/// to listen to time.
/// - [`todos`], a todos tracker inspired by [TodoMVC].
///
/// [The repository has a bunch of examples]: https://github.com/hecrj/iced/tree/0.1/examples
/// [`clock`]: https://github.com/hecrj/iced/tree/0.1/examples/clock
/// [`download_progress`]: https://github.com/hecrj/iced/tree/0.1/examples/download_progress
/// [`events`]: https://github.com/hecrj/iced/tree/0.1/examples/events
/// [`pokedex`]: https://github.com/hecrj/iced/tree/0.1/examples/pokedex
/// [`solar_system`]: https://github.com/hecrj/iced/tree/0.1/examples/solar_system
/// [`stopwatch`]: https://github.com/hecrj/iced/tree/0.1/examples/stopwatch
/// [`todos`]: https://github.com/hecrj/iced/tree/0.1/examples/todos
/// [`Canvas`]: widget/canvas/struct.Canvas.html
/// [PokéAPI]: https://pokeapi.co/
/// [`Subscription`]: type.Subscription.html
/// [TodoMVC]: http://todomvc.com/
///
/// ## A simple "Hello, world!"
///
/// If you just want to get started, here is a simple [`Application`] that
/// says "Hello, world!":
///
/// ```text
/// use iced_baseview::{executor, Application, Command, Element, Settings, Text, WindowQueue};
///
/// pub fn main() -> iced::Result {
///     Hello::run(Settings::default())
/// }
///
/// struct Hello;
///
/// impl Application for Hello {
///     type Executor = executor::Default;
///     type Message = ();
///     type Flags = ();
///
///     fn new(_flags: ()) -> (Hello, Command<Self::Message>) {
///         (Hello, Command::none())
///     }
///
///     fn title(&self) -> String {
///         String::from("A cool application")
///     }
///
///     fn update(&mut self, _window: &mut WindowQueue, _message: Self::Message) -> Command<Self::Message> {
///         Command::none()
///     }
///
///     fn view(&mut self) -> Element<Self::Message> {
///         Text::new("Hello, world!").into()
///     }
/// }
/// ```
pub trait Application: Sized + 'static {
    /// The [`Executor`] that will run commands and subscriptions.
    ///
    /// The [default executor] can be a good starting point!
    ///
    /// [`Executor`]: trait.Executor.html
    /// [default executor]: executor/struct.Default.html
    type Executor: Executor;

    /// The type of __messages__ your [`Application`] will produce.
    ///
    /// [`Application`]: trait.Application.html
    type Message: std::fmt::Debug + Send + 'static + Clone;

    /// The data needed to initialize your [`Application`].
    ///
    /// [`Application`]: trait.Application.html
    type Flags: Send + 'static;

    /// Initializes the [`Application`] with the flags provided to
    /// [`run`] as part of the [`Settings`].
    ///
    /// Here is where you should return the initial state of your app.
    ///
    /// Additionally, you can return a [`Command`](struct.Command.html) if you
    /// need to perform some async action in the background on startup. This is
    /// useful if you want to load state from a file, perform an initial HTTP
    /// request, etc.
    ///
    /// [`Application`]: trait.Application.html
    /// [`run`]: #method.run.html
    /// [`Settings`]: ../settings/struct.Settings.html
    fn new(flags: Self::Flags) -> (Self, Command<Self::Message>);

    /// Handles a __message__ and updates the state of the [`Application`].
    ///
    /// This is where you define your __update logic__. All the __messages__,
    /// produced by either user interactions or commands, will be handled by
    /// this method.
    ///
    /// You can use the given `WindowQueue` to request actions from the `baseview`
    /// window.
    ///
    /// Any [`Command`] returned will be executed immediately in the background.
    ///
    /// [`Application`]: trait.Application.html
    /// [`Command`]: struct.Command.html
    fn update(
        &mut self,
        window: &mut WindowQueue,
        message: Self::Message,
    ) -> Command<Self::Message>;

    /// Returns the event [`Subscription`] for the current state of the
    /// application.
    ///
    /// A [`Subscription`] will be kept alive as long as you keep returning it,
    /// and the __messages__ produced will be handled by
    /// [`update`](#tymethod.update).
    ///
    /// By default, this method returns an empty [`Subscription`].
    ///
    /// [`Subscription`]: struct.Subscription.html
    fn subscription(
        &self,
        _window_subs: &mut WindowSubs<Self::Message>,
    ) -> Subscription<Self::Message> {
        Subscription::none()
    }

    /// Returns the widgets to display in the [`Application`].
    ///
    /// These widgets can produce __messages__ based on user interaction.
    ///
    /// [`Application`]: trait.Application.html
    fn view(&mut self) -> Element<'_, Self::Message>;

    /// Returns the background [`Color`] of the [`Application`].
    ///
    /// By default, it returns [`Color::WHITE`].
    ///
    /// [`Color`]: struct.Color.html
    /// [`Application`]: trait.Application.html
    /// [`Color::WHITE`]: struct.Color.html#const.WHITE
    fn background_color(&self) -> Color {
        Color::WHITE
    }

    /// Returns the [`WindowScalePolicy`] that the [`Application`] should use.
    ///
    /// By default, it returns `WindowScalePolicy::SystemScaleFactor`.
    ///
    /// [`WindowScalePolicy`]: ../settings/enum.WindowScalePolicy.html
    /// [`Application`]: trait.Application.html
    fn scale_policy(&self) -> WindowScalePolicy {
        WindowScalePolicy::SystemScaleFactor
    }

    /// Returns the renderer settings
    #[cfg(feature = "wgpu")]
    fn renderer_settings() -> iced_wgpu::settings::Settings {
        iced_wgpu::settings::Settings {
            // We usually don't want vsync for audio plugins.
            present_mode: iced_wgpu::wgpu::PresentMode::Immediate,
            ..iced_wgpu::settings::Settings::default()
        }
    }

    /// Returns the renderer settings
    #[cfg(feature = "glow")]
    #[cfg(not(feature = "wgpu"))]
    fn renderer_settings(
    ) -> (raw_gl_context::GlConfig, iced_glow::settings::Settings) {
        (
            raw_gl_context::GlConfig::default(),
            iced_glow::settings::Settings::default(),
        )
    }
}
