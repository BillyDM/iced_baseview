use iced_baseview::{
    baseview::{Size, WindowOpenOptions, WindowScalePolicy},
    core::Element,
    settings::IcedBaseviewSettings,
    widget::{button, column, text},
    Application, Center, Renderer, Settings, Task,
};

fn main() {
    let settings = Settings {
        window: WindowOpenOptions {
            title: String::from("iced_baseview hello world"),
            size: Size::new(500.0, 300.0),
            scale: WindowScalePolicy::SystemScaleFactor,
        },
        graphics_settings: iced_graphics::Settings {
            ..Default::default()
        },
        iced_baseview: IcedBaseviewSettings {
            always_redraw: true,
            ..Default::default()
        },
        ..Default::default()
    };

    iced_baseview::open_blocking::<MyProgram>(Flags::default(), settings);
}

#[derive(Default)]
struct Flags {
    initial_value: i64,
}

#[derive(Debug, Clone, Copy)]
enum Message {
    Increment,
    Decrement,
}

struct MyProgram {
    value: i64,
}

impl Application for MyProgram {
    type Message = Message;
    type Flags = Flags;
    type Theme = iced_baseview::Theme;
    type Executor = iced_baseview::executor::Default;

    fn new(flags: Flags) -> (Self, Task<Self::Message>) {
        (
            Self {
                value: flags.initial_value,
            },
            Task::none(),
        )
    }

    fn update(&mut self, message: Self::Message) -> Task<Self::Message> {
        match message {
            Message::Increment => {
                self.value += 1;
            }
            Message::Decrement => {
                self.value -= 1;
            }
        }

        dbg!(self.value);

        Task::none()
    }

    fn view(&self) -> Element<'_, Self::Message, Self::Theme, Renderer> {
        column![
            button("Increment").on_press(Message::Increment),
            text(self.value).size(50),
            button("Decrement").on_press(Message::Decrement)
        ]
        .padding(20)
        .align_x(Center)
        .into()
    }

    fn theme(&self) -> Self::Theme {
        iced_baseview::Theme::Light
    }
}
