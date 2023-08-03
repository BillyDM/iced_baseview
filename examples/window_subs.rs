use iced_baseview::{
    Settings,
    baseview::{Size, WindowOpenOptions, WindowScalePolicy},
    settings::IcedBaseviewSettings,
    widget::Column,
    widget::Container,
    widget::Text,
    widget::Button,
    Application,
    core::{Element, Length, Alignment},
    runtime::{Command, futures::Subscription}, window::WindowSubs,
};
use iced_runtime::window::Action;
use std::time::{Duration, Instant};

static COUNT_INTERVAL: Duration = Duration::from_millis(1000);

fn main() {
    let settings = Settings {
        window: WindowOpenOptions {
            title: String::from("iced_baseview window subscriptions"),
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

#[derive(Debug, Clone, Copy)]
enum Message {
    OnFrame,
    WillClose,
    CloseWindow,
}

struct MyProgram {
    next_interval: Instant,
    count: usize,
}

impl Application for MyProgram {
    type Executor = iced_baseview::executor::Default;
    type Message = Message;
    type Flags = ();
    type Theme = iced_baseview::style::Theme;

    fn new(_flags: ()) -> (Self, Command<Self::Message>) {
        (
            Self {
                next_interval: Instant::now() + COUNT_INTERVAL,
                count: 0,
            },
            Command::none(),
        )
    }

    fn subscription(&self, window_subs: &mut WindowSubs<Message>) -> Subscription<Message> {
        window_subs.on_frame = Some(|| Message::OnFrame);
        window_subs.on_window_will_close = Some(|| Message::WillClose);
        Subscription::none()
    }

    fn update(
        &mut self,
        message: Self::Message,
    ) -> Command<Self::Message> {
        match message {
            Message::OnFrame => {
                let now = Instant::now();
                while now >= self.next_interval {
                    self.next_interval += COUNT_INTERVAL;
                    self.count += 1;
                }
            }
            Message::WillClose => {
                println!("The window will close!");
            }
            Message::CloseWindow => {
                println!("Request to manually close the window.");
                return Command::single(iced_runtime::command::Action::Window(Action::Close))
            }
        }

        Command::none()
    }

    fn view(&self) -> Element<'_, Self::Message, iced_baseview::widget::renderer::Renderer<Self::Theme>> {
        let content = Column::new()
            .width(Length::Fill)
            .align_items(Alignment::Center)
            .push(Text::new(format!("{}", self.count)))
            .push(Button::new(Text::new("Close window")).on_press(Message::CloseWindow));

        Container::new(content)
            .width(Length::Fill)
            .height(Length::Fill)
            .center_x()
            .center_y()
            .into()
    }
    fn title(&self) -> String {
        "Window subs".into()
    }

    fn theme(&self) -> Self::Theme {
        Default::default()
    }
}
