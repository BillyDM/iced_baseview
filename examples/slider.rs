use iced_native::{
    slider, Align, Color, Column, Command, Container, Element, Length, Slider,
    Text,
};

fn main() {
    let settings = iced_baseview::Settings {
        window: iced_baseview::settings::Window {
            title: String::from("iced_baseview slider"),
            size: (500, 300),
            min_size: None,
            max_size: None,
            resizable: false,
        },
    };

    let handle = iced_baseview::Handler::<MyProgram>::open(settings, None);
    handle.app_run_blocking();
}

#[derive(Debug, Copy, Clone)]
pub enum Message {
    SliderChanged(u32),
}

struct MyProgram {
    slider_state: slider::State,
    slider_value: u32,
    slider_value_str: String,
}

impl iced_baseview::Application for MyProgram {
    type AudioToGuiMessage = ();
    type Message = Message;

    fn new() -> Self {
        Self {
            slider_state: slider::State::new(),
            slider_value: 0,
            slider_value_str: String::from("0"),
        }
    }

    fn update(&mut self, message: Self::Message) -> Command<Self::Message> {
        match message {
            Message::SliderChanged(value) => {
                self.slider_value = value;
                self.slider_value_str = format!("{}", value);
            }
        }

        Command::none()
    }

    fn view(&mut self) -> Element<'_, Self::Message, iced_baseview::Renderer> {
        let slider_widget = Slider::new(
            &mut self.slider_state,
            0..=1000,
            self.slider_value,
            Message::SliderChanged,
        );

        let content = Column::new()
            .width(Length::Fill)
            .align_items(Align::Center)
            .padding(20)
            .spacing(20)
            .push(Text::new("Slide me!"))
            .push(slider_widget)
            .push(Text::new(self.slider_value_str.as_str()));

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

    /*
    fn compositor_settings() -> iced_baseview::CompositorSettings {
        iced_baseview::CompositorSettings {
            default_font: None,
            default_text_size: 20,
            antialiasing: Some(iced_baseview::Antialiasing::MSAAx8),
            ..iced_baseview::CompositorSettings::default()
        }
    }
    */
}
