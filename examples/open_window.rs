use std::sync::mpsc;

use iced_native::Command;

fn main() {
    let settings = iced_baseview::Settings {
        window: iced_baseview::settings::Window {
            title: String::from("iced_baseview"),
            size: (500, 300),
            min_size: None,
            max_size: None,
            resizable: true,
        },
        flags: (),
    };

    iced_baseview::Executor::<MyProgram>::run(settings);
}
struct MyProgram {}

impl iced_baseview::Application for MyProgram {
    type Flags = ();
    type AudioToGuiMessage = ();
    type Message = ();

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
    fn new(flags: Self::Flags) -> (Self, Command<Self::Message>) {
        (Self {}, Command::none())
    }
}
