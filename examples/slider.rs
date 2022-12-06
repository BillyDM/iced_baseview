use iced_baseview::{
    baseview::{Size, WindowOpenOptions, WindowScalePolicy},
    executor, open_blocking,
    settings::IcedBaseviewSettings,
    widget::Column,
    widget::Container,
    widget::Slider,
    widget::Text,
    window::WindowQueue,
    Alignment, Application, Command, Element, Length, Settings,
};

fn main() {
    let settings = Settings {
        window: WindowOpenOptions {
            title: String::from("iced_baseview slider"),
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
    SliderChanged(u32),
}

struct MyProgram {
    slider_value: u32,
    slider_value_str: String,
}

impl Application for MyProgram {
    type Executor = executor::Default;
    type Message = Message;
    type Flags = ();
    type Theme = iced_baseview::renderer::Theme;

    fn new(_flags: ()) -> (Self, Command<Self::Message>) {
        (
            Self {
                slider_value: 0,
                slider_value_str: String::from("0"),
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
            Message::SliderChanged(value) => {
                self.slider_value = value;
                self.slider_value_str = format!("{}", value);
            }
        }

        Command::none()
    }

    fn view(&self) -> Element<'_, Self::Message, Self::Theme> {
        let slider_widget = Slider::new(0..=1000, self.slider_value, Message::SliderChanged);

        let content = Column::new()
            .width(Length::Fill)
            .align_items(Alignment::Center)
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

    fn title(&self) -> String {
        "Slider".into()
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
