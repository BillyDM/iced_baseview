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

struct MyProgram;

impl Application for MyProgram {
    type Executor = iced_baseview::executor::Default;
    type Message = ();
    type Theme = iced_baseview::style::Theme;
    type Flags = ();

    fn new(_flags: ()) -> (Self, Command<Self::Message>) {
        (Self {}, Command::none())
    }
    fn title(&self) -> String {
        "Hello World!".into()
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
            .push(Text::new("Hello World!"))
            .push(Rule::horizontal(10));

        Container::new(content)
            .width(Length::Fill)
            .height(Length::Fill)
            .center_x()
            .center_y()
            .into()
    }

    fn renderer_settings() -> iced_renderer::Settings {
        iced_renderer::Settings {
            antialiasing: Some(iced_graphics::Antialiasing::MSAAx4),
            ..Default::default()
        }
    }
}
