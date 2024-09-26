use iced_baseview::{
    baseview::{Size, WindowOpenOptions, WindowScalePolicy},
    widget::{Column, Container, Rule, Text},
    Alignment, Application, Element, Length, Renderer, Settings, Task, Theme,
};

fn main() {
    let settings = Settings {
        window: WindowOpenOptions {
            title: String::from("iced_baseview hello world"),
            size: Size::new(500.0, 300.0),
            scale: WindowScalePolicy::SystemScaleFactor,
        },
        ..Default::default()
    };

    iced_baseview::open_blocking::<MyProgram>((), settings);
}

struct MyProgram;

impl Application for MyProgram {
    type Message = ();
    type Flags = ();
    type Theme = Theme;
    type Executor = iced_baseview::executor::Default;

    fn new(_flags: ()) -> (Self, Task<Self::Message>) {
        (Self {}, Task::none())
    }

    fn title(&self) -> String {
        "Hello World!".into()
    }

    fn update(&mut self, _message: Self::Message) -> Task<Self::Message> {
        Task::none()
    }

    fn view(&self) -> Element<'_, Self::Message, Self::Theme, Renderer> {
        let content = Column::new()
            .width(Length::Fill)
            .align_x(Alignment::Center)
            .push(Text::new("Hello World!"))
            .push(Rule::horizontal(10));

        Container::new(content)
            .width(Length::Fill)
            .height(Length::Fill)
            .center(Length::Fill)
            .into()
    }

    fn theme(&self) -> Self::Theme {
        Theme::Dark
    }
}
