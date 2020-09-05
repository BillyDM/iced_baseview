use iced_native::{Column, Command, Container, Element};

fn main() {
    let settings = iced_baseview::Settings {
        window: iced_baseview::settings::Window {
            title: String::from("iced_baseview open window"),
            size: (500, 300),
            min_size: None,
            max_size: None,
            resizable: true,
        },
    };

    iced_baseview::Executor::<MyProgram>::run(settings);
}
struct MyProgram {}

impl iced_baseview::Application for MyProgram {
    type AudioToGuiMessage = ();
    type Message = ();

    fn new() -> (Self, Command<Self::Message>) {
        (Self {}, Command::none())
    }

    fn update(&mut self, _message: Self::Message) -> Command<Self::Message> {
        Command::none()
    }

    fn view(&mut self) -> Element<'_, Self::Message, iced_wgpu::Renderer> {
        let content = Column::new();

        Container::new(content).into()
    }
}
