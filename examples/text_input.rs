use baseview::{Size, WindowOpenOptions, WindowScalePolicy};
use iced_baseview::{
    executor, text_input, Alignment, Application, Color, Column, Command,
    Container, Element, IcedBaseviewSettings, IcedWindow, Length, Settings,
    Text, TextInput, WindowQueue,
};

fn main() {
    let settings = Settings {
        window: WindowOpenOptions {
            title: String::from("iced_baseview text input"),
            size: Size::new(500.0, 300.0),
            scale: WindowScalePolicy::SystemScaleFactor,

            // FIXME: The current glow_glpyh version does not enable the correct extension in their
            //        shader so this currently won't work with OpenGL <= 3.2
            #[cfg(feature = "glow")]
            #[cfg(not(feature = "wgpu"))]
            gl_config: Some(baseview::gl::GlConfig {
                version: (3, 3),
                ..baseview::gl::GlConfig::default()
            }),
        },
        iced_baseview: IcedBaseviewSettings {
            ignore_non_modifier_keys: false,
            always_redraw: true,
        },
        flags: (),
    };

    IcedWindow::<MyProgram>::open_blocking(settings);
}

#[derive(Debug, Clone)]
pub enum Message {
    TextInputChanged(String),
}

struct MyProgram {
    text_input_state: text_input::State,
    text_input_str: String,
}

impl Application for MyProgram {
    type Executor = executor::Default;
    type Message = Message;
    type Flags = ();

    fn new(_flags: ()) -> (Self, Command<Self::Message>) {
        (
            Self {
                text_input_state: text_input::State::new(),
                text_input_str: String::from(""),
            },
            Command::none(),
        )
    }

    fn update(
        &mut self,
        _window: &mut WindowQueue,
        message: Self::Message,
    ) -> Command<Self::Message> {
        match message {
            Message::TextInputChanged(value) => {
                self.text_input_str = value;
            }
        }

        Command::none()
    }

    fn view(&mut self) -> Element<'_, Self::Message> {
        let text_input_widget = TextInput::new(
            &mut self.text_input_state,
            "Hello!",
            &self.text_input_str,
            Message::TextInputChanged,
        );

        let content = Column::new()
            .width(Length::Fill)
            .align_items(Alignment::Center)
            .padding(20)
            .spacing(20)
            .push(Text::new("Write text!"))
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

    /*
    fn renderer_settings() -> renderer::Settings {
        renderer::Settings {
            default_font: None,
            default_text_size: 20,
            antialiasing: Some(renderer::Antialiasing::MSAAx4),
            ..renderer::Settings::default()
        }
    }
    */
}
