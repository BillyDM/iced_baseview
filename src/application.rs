use crate::renderer;

use iced_native::{Color, Command, Element};

pub trait Application: Sized {
    type AudioToGuiMessage;

    /// The type of __messages__ your [`Program`] will produce.
    ///
    /// [`Program`]: trait.Program.html
    type Message: std::fmt::Debug + Send;

    /// Initializes the [`Application`] with the flags provided to
    /// [`run`] as part of the [`Settings`].
    ///
    /// Here is where you should return the initial state of your app.
    ///
    /// [`Application`]: trait.Application.html
    /// [`run`]: #method.run.html
    /// [`Settings`]: ../settings/struct.Settings.html
    fn new() -> Self;

    /// Handles a __message__ and updates the state of the [`Program`].
    ///
    /// This is where you define your __update logic__. All the __messages__,
    /// produced by either user interactions or commands, will be handled by
    /// this method.
    ///
    /// Any [`Command`] returned will be executed immediately in the
    /// background by shells.
    ///
    /// [`Program`]: trait.Application.html
    /// [`Command`]: struct.Command.html
    fn update(&mut self, message: Self::Message) -> Command<Self::Message>;

    /// Returns the widgets to display in the [`Program`].
    ///
    /// These widgets can produce __messages__ based on user interaction.
    ///
    /// [`Program`]: trait.Program.html
    fn view(&mut self) -> Element<'_, Self::Message, renderer::Renderer>;

    /// Returns the background color of the application.
    fn background_color(&self) -> Color {
        Color::WHITE
    }

    /// Returns the scale factor of the [`Application`].
    ///
    /// It can be used to dynamically control the size of the UI at runtime
    /// (i.e. zooming).
    ///
    /// For instance, a scale factor of `2.0` will make widgets twice as big,
    /// while a scale factor of `0.5` will shrink them to half their size.
    ///
    /// By default, it returns `1.0`.
    ///
    /// [`Application`]: trait.Application.html
    fn scale_factor(&self) -> f64 {
        1.0
    }

    /// Returns the renderer settings
    fn renderer_settings() -> renderer::Settings {
        renderer::Settings::default()
    }
}
