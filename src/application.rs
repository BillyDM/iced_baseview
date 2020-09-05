use iced_native::{Command, Program};

pub trait Application: Program {
    type Flags;
    type AudioToGuiMessage;

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
    fn new(flags: &Self::Flags) -> (Self, Command<Self::Message>);
}
