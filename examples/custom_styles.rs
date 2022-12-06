use iced_baseview::{
    baseview::{Size, WindowOpenOptions, WindowScalePolicy},
    open_blocking,
    settings::IcedBaseviewSettings,
    widget::{container, rule, text, Column, Container, Rule, Text},
    window::WindowQueue,
    Alignment, Application, Color, Command, Element, Length, Settings,
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

#[derive(Default)]
enum Theme {
    #[default]
    Light,
}

impl iced_native::application::StyleSheet for Theme {
    type Style = ();

    fn appearance(&self, _style: &Self::Style) -> iced_native::application::Appearance {
        iced_native::application::Appearance {
            text_color: Color::BLACK,
            background_color: Color::WHITE,
        }
    }
}

impl container::StyleSheet for Theme {
    type Style = ();

    fn appearance(&self, _style: &Self::Style) -> container::Appearance {
        Default::default()
    }
}

#[derive(Clone, Copy, Default)]
enum TextStyle {
    #[default]
    Green,
    Black,
}

impl text::StyleSheet for Theme {
    type Style = TextStyle;

    fn appearance(&self, style: Self::Style) -> text::Appearance {
        let color = match style {
            Self::Style::Green => Color::from_rgb(0.25, 0.75, 0.25),
            Self::Style::Black => Color::BLACK,
        };
        text::Appearance { color: Some(color) }
    }
}

impl rule::StyleSheet for Theme {
    type Style = ();

    fn appearance(&self, _style: &Self::Style) -> rule::Appearance {
        rule::Appearance {
            color: Color::from_rgb(0.75, 0.75, 0.75),
            width: 1,
            radius: 0.0,
            fill_mode: rule::FillMode::Percent(50.0),
        }
    }
}

struct MyProgram {}

impl Application for MyProgram {
    type Executor = iced_futures::backend::default::Executor;
    type Message = ();
    type Flags = ();
    type Theme = Theme;

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

    fn view(&self) -> Element<'_, Self::Message, Self::Theme> {
        let content = Column::new()
            .width(Length::Fill)
            .align_items(Alignment::Center)
            .push(Text::new("Hello World!").style(TextStyle::Black))
            .push(Rule::horizontal(10));

        Container::new(content)
            .width(Length::Fill)
            .height(Length::Fill)
            .center_x()
            .center_y()
            .into()
    }

    fn title(&self) -> String {
        "Hello World!".into()
    }

    fn theme(&self) -> Self::Theme {
        Theme::Light
    }
}
