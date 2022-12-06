use baseview::{Size, WindowOpenOptions, WindowScalePolicy};
use iced_baseview::{
    executor, open_blocking, settings::IcedBaseviewSettings, widget::Button, widget::Column,
    widget::Container, widget::Text, window::WindowQueue, window::WindowSubs, Alignment,
    Application, Command, Element, Length, Settings, Subscription,
};
use std::time::{Duration, Instant};

static COUNT_INTERVAL: Duration = Duration::from_millis(1000);

fn main() {
    let settings = Settings {
        window: WindowOpenOptions {
            title: String::from("iced_baseview window subscriptions"),
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
    type Executor = executor::Default;
    type Message = Message;
    type Flags = ();
    type Theme = iced_baseview::renderer::Theme;

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

    fn view(&self) -> Element<'_, Self::Message, Self::Theme> {
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
