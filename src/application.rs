//! Create interactive, native cross-platform plugin applications.

use crate::{
    mouse, Clipboard, Color, Command, Debug, Executor, Runtime, Settings, Size, Subscription,
};

use iced_native::program::Program;

pub trait Application: Program {
    /// The data needed to initialize your [`Application`].
    ///
    /// [`Application`]: trait.Application.html
    type Flags;

    /// Initializes the [`Application`] with the settings and flags provided.
    ///
    /// Here is where you should return the initial state of your GUI.
    ///
    /// Additionally, you can return a [`Command`](struct.Command.html) if you
    /// need to perform some async action in the background on startup. This is
    /// useful if you want to load state from a file, perform an initial HTTP
    /// request, etc.
    ///
    /// [`Application`]: trait.Application.html
    fn new(settings: &Settings<Self::Flags>) -> (Self, Command<Self::Message>);

    /// Returns the event `Subscription` for the current state of the
    /// application.
    ///
    /// The messages produced by the `Subscription` will be handled by
    /// [`update`](#tymethod.update).
    ///
    /// A `Subscription` will be kept alive as long as you keep returning it!
    ///
    /// By default, it returns an empty subscription.
    fn subscription(&self) -> Subscription<Self::Message> {
        Subscription::none()
    }

    /// Returns the background [`Color`] of the [`Application`].
    ///
    /// By default, it returns [`Color::BLACK`].
    ///
    /// [`Color`]: struct.Color.html
    /// [`Application`]: trait.Application.html
    /// [`Color::BLACK`]: struct.Color.html#const.BLACK
    fn background_color(&self) -> Color {
        Color::BLACK
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
}

/*  Implementation of `Program` from
https://github.com/hecrj/iced/blob/master/native/src/program.rs:

//! Build interactive programs using The Elm Architecture.
use crate::{Command, Element, Renderer};

mod state;

pub use state::State;

/// The core of a user interface application following The Elm Architecture.
pub trait Program: Sized {
    /// The graphics backend to use to draw the [`Program`].
    ///
    /// [`Program`]: trait.Program.html
    type Renderer: Renderer;

    /// The type of __messages__ your [`Program`] will produce.
    ///
    /// [`Program`]: trait.Program.html
    type Message: std::fmt::Debug + Send;

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
    fn view(&mut self) -> Element<'_, Self::Message, Self::Renderer>;
}
*/
