use iced_native::{
    Align, Color, Column, Command, Container, Element, Length, Rule, Text,
};

fn main() {
    let settings = iced_baseview::Settings {
        window: iced_baseview::settings::Window {
            title: String::from("iced_baseview hello world"),
            size: (500, 300),
            min_size: None,
            max_size: None,
            resizable: false,
        },
    };

    iced_baseview::Handler::<MyProgram>::run(settings);
}
struct MyProgram {}

impl iced_baseview::Application for MyProgram {
    type AudioToGuiMessage = ();
    type Message = ();

    fn new() -> Self {
        Self {}
    }

    fn update(&mut self, _message: Self::Message) -> Command<Self::Message> {
        Command::none()
    }

    fn view(&mut self) -> Element<'_, Self::Message, iced_baseview::Renderer> {
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

    fn background_color() -> Color {
        Color::WHITE
    }

    fn compositor_settings() -> iced_baseview::CompositorSettings {
        iced_baseview::CompositorSettings {
            default_font: None,
            default_text_size: 20,
            antialiasing: Some(iced_baseview::Antialiasing::MSAAx8),
            ..iced_baseview::CompositorSettings::default()
        }
    }
}
