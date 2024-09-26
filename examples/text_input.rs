use iced_baseview::{
    baseview::{Size, WindowOpenOptions, WindowScalePolicy},
    settings::IcedBaseviewSettings,
    widget::{checkbox, column, text, text_input},
    Application, Center, Element, Fill, Font, Pixels, Renderer, Settings, Task, Theme,
};

fn main() {
    let settings = Settings {
        window: WindowOpenOptions {
            title: String::from("iced_baseview counter demo"),
            size: Size::new(500.0, 420.0),
            scale: WindowScalePolicy::SystemScaleFactor,
        },
        graphics_settings: iced_graphics::Settings {
            ..Default::default()
        },
        iced_baseview: IcedBaseviewSettings {
            ..Default::default()
        },
        ..Default::default()
    };

    iced_baseview::open_blocking::<MyProgram>((), settings);
}

#[derive(Debug, Clone)]
enum Message {
    InputChanged(String),
    ToggleSecureInput(bool),
    ToggleTextInputIcon(bool),
}

struct MyProgram {
    input_value: String,
    input_is_secure: bool,
    input_is_showing_icon: bool,
}

impl Application for MyProgram {
    type Message = Message;
    type Flags = ();
    type Theme = Theme;
    type Executor = iced_baseview::executor::Default;

    fn new(_flags: Self::Flags) -> (Self, Task<Self::Message>) {
        (
            Self {
                input_value: String::new(),
                input_is_secure: false,
                input_is_showing_icon: false,
            },
            Task::none(),
        )
    }

    fn update(&mut self, message: Self::Message) -> Task<Self::Message> {
        match message {
            Message::InputChanged(input_value) => {
                self.input_value = input_value;
            }
            Message::ToggleSecureInput(is_secure) => {
                self.input_is_secure = is_secure;
            }
            Message::ToggleTextInputIcon(show_icon) => {
                self.input_is_showing_icon = show_icon;
            }
        }

        Task::none()
    }

    fn view(&self) -> Element<'_, Self::Message, Self::Theme, Renderer> {
        let value = &self.input_value;
        let is_secure = self.input_is_secure;
        let is_showing_icon = self.input_is_showing_icon;

        let mut text_input = text_input("Type something to continue...", value)
            .on_input(Message::InputChanged)
            .padding(10)
            .size(30);

        if is_showing_icon {
            text_input = text_input.icon(text_input::Icon {
                font: Font::default(),
                code_point: '🚀',
                size: Some(Pixels(28.0)),
                spacing: 10.0,
                side: text_input::Side::Right,
            });
        }

        let container = column![text("Text Input").size(50)].padding(20).spacing(20);

        container
            .push("Use a text input to ask for different kinds of information.")
            .push(text_input.secure(is_secure))
            .push(checkbox("Enable password mode", is_secure).on_toggle(Message::ToggleSecureInput))
            .push(checkbox("Show icon", is_showing_icon).on_toggle(Message::ToggleTextInputIcon))
            .push(
                "A text input produces a message every time it changes. It is \
                 very easy to keep track of its contents:",
            )
            .push(
                text(if value.is_empty() {
                    "You have not typed anything yet..."
                } else {
                    value
                })
                .width(Fill)
                .align_x(Center),
            )
            .into()
    }

    fn theme(&self) -> Self::Theme {
        Theme::Dark
    }
}
