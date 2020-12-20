use iced_baseview::{
    executor, renderer, settings, slider, text_input, Align, Application,
    Color, Column, Command, Container, Element, Length, Parent, Runner,
    Settings, Slider, Text, TextInput, WindowScalePolicy,
};

fn main() {
    let settings = Settings {
        window: settings::Window {
            title: String::from("iced_baseview slider"),
            logical_size: (500, 300),
            scale_policy: WindowScalePolicy::SystemScaleFactor,
        },
        flags: (),
    };

    let (_, opt_app_runner) = Runner::<MyProgram>::open(settings, Parent::None);

    opt_app_runner.unwrap().app_run_blocking();
}

#[derive(Debug, Clone)]
pub enum Message {
    SliderChanged(u32),
    TextInputChanged(String),
}

struct MyProgram {
    slider_state: slider::State,
    text_input_state: text_input::State,
    slider_value: u32,
    slider_value_str: String,
    text_input_string: String,
}

impl Application for MyProgram {
    type Executor = executor::Default;
    type Message = Message;
    type Flags = ();

    fn new(_flags: ()) -> (Self, Command<Self::Message>) {
        (
            Self {
                slider_state: slider::State::new(),
                text_input_state: text_input::State::new(),
                slider_value: 0,
                slider_value_str: String::from("0"),
                text_input_string: String::from(""),
            },
            Command::none(),
        )
    }

    fn update(&mut self, message: Self::Message) -> Command<Self::Message> {
        match message {
            Message::SliderChanged(value) => {
                self.slider_value = value;
                self.slider_value_str = format!("{}", value);
            }
            Message::TextInputChanged(value) => {
                self.text_input_string = value;
            }
        }

        Command::none()
    }

    fn view(&mut self) -> Element<'_, Self::Message> {
        let slider_widget = Slider::new(
            &mut self.slider_state,
            0..=1000,
            self.slider_value,
            Message::SliderChanged,
        );

        let text_input_widget = TextInput::new(
            &mut self.text_input_state,
            "Hello!",
            &self.text_input_string,
            Message::TextInputChanged,
        );

        let content = Column::new()
            .width(Length::Fill)
            .align_items(Align::Center)
            .padding(20)
            .spacing(20)
            .push(Text::new("Slide me!"))
            .push(slider_widget)
            .push(Text::new(self.slider_value_str.as_str()))
            .push(text_input_widget);

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

    fn renderer_settings() -> renderer::Settings {
        renderer::Settings {
            default_font: None,
            default_text_size: 20,
            antialiasing: Some(renderer::Antialiasing::MSAAx4),
            ..renderer::Settings::default()
        }
    }
}
