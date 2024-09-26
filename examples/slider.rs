use iced_baseview::{
    baseview::{Size, WindowOpenOptions, WindowScalePolicy},
    settings::IcedBaseviewSettings,
    widget::{column, container, slider, text, vertical_slider},
    Application, Center, Element, Fill, Renderer, Settings, Task, Theme,
};

fn main() {
    let settings = Settings {
        window: WindowOpenOptions {
            title: String::from("iced_baseview slider demo"),
            size: Size::new(500.0, 320.0),
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

#[derive(Debug, Clone, Copy)]
enum Message {
    SliderChanged(u8),
}

struct MyProgram {
    value: u8,
}

impl Application for MyProgram {
    type Message = Message;
    type Flags = ();
    type Theme = Theme;
    type Executor = iced_baseview::executor::Default;

    fn new(_flags: Self::Flags) -> (Self, Task<Self::Message>) {
        (Self { value: 0 }, Task::none())
    }

    fn update(&mut self, message: Self::Message) -> Task<Self::Message> {
        match message {
            Message::SliderChanged(value) => {
                self.value = value;
            }
        }

        Task::none()
    }

    fn view(&self) -> Element<'_, Self::Message, Self::Theme, Renderer> {
        let h_slider = container(
            slider(1..=100, self.value, Message::SliderChanged)
                .default(50)
                .shift_step(5),
        )
        .width(250);

        let v_slider = container(
            vertical_slider(1..=100, self.value, Message::SliderChanged)
                .default(50)
                .shift_step(5),
        )
        .height(200);

        let text = text(self.value);

        column![v_slider, h_slider, text,]
            .width(Fill)
            .align_x(Center)
            .spacing(20)
            .padding(20)
            .into()
    }

    fn theme(&self) -> Self::Theme {
        Theme::Dark
    }
}
