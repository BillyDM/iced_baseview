use baseview::{Size, WindowOpenOptions, WindowScalePolicy};
use iced_baseview::{
    executor, open_blocking, settings::IcedBaseviewSettings, widget::Column, widget::Container,
    widget::Text, widget::TextInput, window::WindowQueue, Alignment, Application, Command, Element,
    Length, Settings,
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

    open_blocking::<MyProgram>(settings);
}

#[derive(Debug, Clone)]
pub enum Message {
    TextInputChanged(String),
}

struct MyProgram {
    text_input_str: String,
}

impl Application for MyProgram {
    type Executor = executor::Default;
    type Message = Message;
    type Flags = ();
    type Theme = iced_baseview::renderer::Theme;

    fn new(_flags: ()) -> (Self, Command<Self::Message>) {
        (
            Self {
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

    fn view(&self) -> Element<'_, Self::Message, Self::Theme> {
        let text_input_widget =
            TextInput::new("Hello!", &self.text_input_str, Message::TextInputChanged);

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

    fn title(&self) -> String {
        "Text input".into()
    }

    fn theme(&self) -> Self::Theme {
        Default::default()
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
