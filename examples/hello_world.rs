use iced_baseview::{
    executor, settings, Align, Application, Color, Column, Command, Container,
    Element, Length, Parent, Rule, Runner, Settings, Text, WindowScalePolicy,
};

fn main() {
    let settings = Settings {
        window: settings::Window {
            title: String::from("iced_baseview hello world"),
            logical_size: (500, 300),
            scale_policy: WindowScalePolicy::SystemScaleFactor,
        },
        flags: (),
    };

    let opt_app_runner = Runner::<MyProgram>::open(settings, Parent::None);

    opt_app_runner.unwrap().app_run_blocking();
}

struct MyProgram {}

impl Application for MyProgram {
    type Executor = executor::Default;
    type Message = ();
    type Flags = ();

    fn new(_flags: ()) -> (Self, Command<Self::Message>) {
        (Self {}, Command::none())
    }

    fn update(&mut self, _message: Self::Message) -> Command<Self::Message> {
        Command::none()
    }

    fn view(&mut self) -> Element<'_, Self::Message> {
        let content = Column::new()
            .width(Length::Fill)
            .align_items(Align::Center)
            .push(Text::new("Hello World!"))
            .push(Rule::horizontal(10));

        Container::new(content)
            .width(Length::Fill)
            .height(Length::Fill)
            .center_x()
            .center_y()
            .into()
    }

    fn background_color(&self) -> Color {
        Color::WHITE
    }
}
