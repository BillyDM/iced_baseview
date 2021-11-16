use baseview::{Size, WindowOpenOptions, WindowScalePolicy};
use iced_baseview::{
    button, executor, Align, Application, Button, Color, Column, Command,
    Container, Element, IcedWindow, Length, Settings, Subscription, Text,
    WindowQueue, WindowSubs,
};
use std::time::{Duration, Instant};

static COUNT_INTERVAL: Duration = Duration::from_millis(1000);

fn main() {
    let settings = Settings {
        window: WindowOpenOptions {
            title: String::from("iced_baseview window subscriptions"),
            size: Size::new(500.0, 300.0),
            scale: WindowScalePolicy::SystemScaleFactor,
        },
        flags: (),
    };

    IcedWindow::<MyProgram>::open_blocking(settings);
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

    close_btn_state: button::State,
}

impl Application for MyProgram {
    type Executor = executor::Default;
    type Message = Message;
    type Flags = ();

    fn new(_flags: ()) -> (Self, Command<Self::Message>) {
        (
            Self {
                next_interval: Instant::now() + COUNT_INTERVAL,
                count: 0,

                close_btn_state: button::State::new(),
            },
            Command::none(),
        )
    }

    fn subscription(
        &self,
        window_subs: &mut WindowSubs<Message>,
    ) -> Subscription<Message> {
        window_subs.on_frame = Some(Message::OnFrame);
        window_subs.on_window_will_close = Some(Message::WillClose);
        Subscription::none()
    }

    fn update(
        &mut self,
        window: &mut WindowQueue,
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
                window.close_window().unwrap();
            }
        }

        Command::none()
    }

    fn view(&mut self) -> Element<'_, Self::Message> {
        let content = Column::new()
            .width(Length::Fill)
            .align_items(Align::Center)
            .push(Text::new(format!("{}", self.count)))
            .push(
                Button::new(
                    &mut self.close_btn_state,
                    Text::new("Close window"),
                )
                .on_press(Message::CloseWindow),
            );

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
