use iced_native::{
    Align, Color, Column, Command, Container, Element, Length, Rule, Text,
};

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

impl iced_native::Program for MyProgram {
    type Message = ();
    // Rendering backend chosen here by the user
    type Renderer = iced_wgpu::Renderer;

    fn update(&mut self, _message: Self::Message) -> Command<Self::Message> {
        Command::none()
    }

    fn view(&mut self) -> Element<'_, Self::Message, iced_wgpu::Renderer> {
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
}

impl iced_baseview::Application for MyProgram {
    type AudioToGuiMessage = ();
    type Compositor = iced_wgpu::window::Compositor;

    fn new() -> (Self, Command<Self::Message>) {
        (Self {}, Command::none())
    }

    fn background_color() -> Color {
        Color::WHITE
    }

    fn compositor_settings() -> iced_wgpu::Settings {
        iced_wgpu::Settings {
            default_font: None,
            default_text_size: 20,
            antialiasing: Some(iced_wgpu::Antialiasing::MSAAx8),
            ..iced_wgpu::Settings::default()
        }
    }
}
