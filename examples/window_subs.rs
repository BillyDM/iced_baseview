use iced_baseview::{
    baseview::{Size, WindowOpenOptions, WindowScalePolicy},
    core::{Alignment, Element, Length},
    futures::Subscription,
    widget::{Button, Column, Container, Space, Text},
    window, Application, Renderer, Settings, Task, Theme, WindowSubs,
};
use std::{
    sync::Arc,
    time::{Duration, Instant},
};

static COUNT_INTERVAL: Duration = Duration::from_millis(1000);

fn main() {
    let settings = Settings {
        window: WindowOpenOptions {
            title: String::from("iced_baseview window subscriptions"),
            size: Size::new(500.0, 300.0),
            scale: WindowScalePolicy::SystemScaleFactor,
        },
        ..Default::default()
    };

    iced_baseview::open_blocking::<MyProgram>((), settings);
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
    type Message = Message;
    type Flags = ();
    type Theme = Theme;
    type Executor = iced_baseview::executor::Default;

    fn new(_flags: ()) -> (Self, Task<Self::Message>) {
        (
            Self {
                next_interval: Instant::now() + COUNT_INTERVAL,
                count: 0,
            },
            Task::none(),
        )
    }

    fn subscription(&self, window_subs: &mut WindowSubs<Message>) -> Subscription<Message> {
        window_subs.on_frame = Some(Arc::new(|| Some(Message::OnFrame)));
        window_subs.on_window_will_close = Some(Arc::new(|| Some(Message::WillClose)));
        Subscription::none()
    }

    fn update(&mut self, message: Self::Message) -> Task<Self::Message> {
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
                return window::close();
            }
        }

        Task::none()
    }

    fn view(&self) -> Element<'_, Self::Message, Self::Theme, Renderer> {
        let content = Column::new()
            .width(Length::Fill)
            .align_x(Alignment::Center)
            .push(Text::new(format!("{}", self.count)).size(50))
            .push(Space::new(20.0, 20.0))
            .push(Button::new(Text::new("Close window")).on_press(Message::CloseWindow));

        Container::new(content)
            .width(Length::Fill)
            .height(Length::Fill)
            .center(Length::Fill)
            .into()
    }

    fn theme(&self) -> Self::Theme {
        Theme::Dark
    }
}
