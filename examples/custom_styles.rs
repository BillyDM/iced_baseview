use iced_baseview::{
    baseview::{Size, WindowOpenOptions, WindowScalePolicy},
    core::{Alignment, Element, Length},
    runtime::Command,
    settings::IcedBaseviewSettings,
    widget::Column,
    widget::Container,
    widget::Rule,
    widget::Text,
    Application, Settings,
};
use iced_graphics::core::{BorderRadius, Color};

fn main() {
    let settings = Settings {
        window: WindowOpenOptions {
            title: String::from("iced_baseview hello world"),
            size: Size::new(500.0, 300.0),
            scale: WindowScalePolicy::SystemScaleFactor,
        },
        iced_baseview: IcedBaseviewSettings {
            ignore_non_modifier_keys: false,
            always_redraw: true,
        },
        flags: (),
    };

    MyProgram::open_blocking(settings);
}

#[derive(Default)]
enum Theme {
    #[default]
    Light,
}

impl iced_baseview::style::application::StyleSheet for Theme {
    type Style = ();

    fn appearance(&self, _style: &Self::Style) -> iced_baseview::style::application::Appearance {
        iced_baseview::style::application::Appearance {
            text_color: Color::BLACK,
            background_color: Color::WHITE,
        }
    }
}

impl iced_baseview::style::container::StyleSheet for Theme {
    type Style = ();

    fn appearance(&self, _style: &Self::Style) -> iced_baseview::style::container::Appearance {
        Default::default()
    }
}

impl iced_baseview::style::rule::StyleSheet for Theme {
    type Style = ();

    fn appearance(&self, _style: &Self::Style) -> iced_baseview::style::rule::Appearance {
        iced_baseview::style::rule::Appearance {
            color: Color::from_rgb(0.0, 0.5, 0.0),
            width: 1,
            radius: BorderRadius::default(),
            fill_mode: iced_baseview::style::rule::FillMode::Percent(50.0),
        }
    }
}

#[derive(Default, Clone)]
enum TextStyle {
    #[default]
    Green,
}

impl iced_baseview::widget::text::StyleSheet for Theme {
    type Style = TextStyle;

    fn appearance(&self, style: Self::Style) -> iced_widget::text::Appearance {
        match style {
            TextStyle::Green => iced_widget::text::Appearance {
                color: Some(Color::from_rgb(0.0, 0.5, 0.0)),
            },
        }
    }
}

struct MyProgram {}

impl Application for MyProgram {
    type Executor = iced_baseview::executor::Default;
    type Message = ();
    type Flags = ();
    type Theme = Theme;

    fn new(_flags: ()) -> (Self, Command<Self::Message>) {
        (Self {}, Command::none())
    }

    fn update(&mut self, _message: Self::Message) -> Command<Self::Message> {
        Command::none()
    }

    fn view(
        &self,
    ) -> Element<'_, Self::Message, iced_baseview::widget::renderer::Renderer<Self::Theme>> {
        let content = Column::new()
            .width(Length::Fill)
            .align_items(Alignment::Center)
            .push(Text::new("Hello World!").style(TextStyle::Green))
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
