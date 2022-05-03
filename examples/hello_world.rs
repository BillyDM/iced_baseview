use baseview::{Size, WindowOpenOptions, WindowScalePolicy};
use iced_baseview::{
    executor, Alignment, Application, Color, Column, Command, Container,
    Element, IcedBaseviewSettings, IcedWindow, Length, Rule, Settings, Text,
    WindowQueue,
};

fn main() {
    let settings = Settings {
        window: WindowOpenOptions {
            title: String::from("iced_baseview hello world"),
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

struct MyProgram {}

impl Application for MyProgram {
    type Executor = executor::Default;
    type Message = ();
    type Flags = ();

    fn new(_flags: ()) -> (Self, Command<Self::Message>) {
        (Self {}, Command::none())
    }

    fn update(
        &mut self,
        _window: &mut WindowQueue,
        _message: Self::Message,
    ) -> Command<Self::Message> {
        Command::none()
    }

    fn view(&mut self) -> Element<'_, Self::Message> {
        let content = Column::new()
            .width(Length::Fill)
            .align_items(Alignment::Center)
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
