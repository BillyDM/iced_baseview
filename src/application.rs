use iced_native::{Color, Command, Program};

pub trait Application: Program {
    type AudioToGuiMessage;
    type Compositor: iced_graphics::window::Compositor<Renderer = Self::Renderer>
        + 'static;

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
    fn new() -> (Self, Command<Self::Message>);

    fn background_color() -> Color {
        Color::WHITE
    }

    fn compositor_settings(
    ) -> <Self::Compositor as iced_graphics::window::Compositor>::Settings {
        Default::default()
    }
}
