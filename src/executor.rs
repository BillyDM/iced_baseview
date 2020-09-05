use crate::{Application, Settings};

use std::sync::mpsc;

use baseview::{Event, WindowInfo};
use iced_native::Command;

pub struct Executor<A: Application> {
    application: A,
    initial_command: Command<A::Message>,
}

impl<A: Application> Executor<A> {
    pub fn run(settings: Settings<A::Flags>) {
        // Initialize user's application.
        let (application, initial_command) = A::new(settings.flags);
        let mut executor = Self {
            application,
            initial_command,
        };

        let window_open_options = baseview::WindowOpenOptions {
            title: settings.window.title.as_str(),
            width: settings.window.size.0 as usize,
            height: settings.window.size.1 as usize,
            parent: baseview::Parent::None,
        };

        // Create channel for sending messages from audio to GUI.
        let (_app_message_tx, app_message_rx) = mpsc::channel::<A::AudioToGuiMessage>();

        // Run the baseview window with the executor.
        let _ = baseview::Window::open(window_open_options, executor, app_message_rx);
    }
}

impl<A: Application> baseview::Application for Executor<A> {
    type AppMessage = A::AudioToGuiMessage;

    fn create_context(
        &mut self,
        window: raw_window_handle::RawWindowHandle,
        window_info: &WindowInfo,
    ) {
    }

    fn on_event(&mut self, event: Event) {
        match event {
            Event::RenderExpose => {}
            Event::CursorMotion(x, y) => {
                println!("Cursor moved, x: {}, y: {}", x, y);
            }
            Event::MouseDown(button_id) => {
                println!("Mouse down, button id: {:?}", button_id);
            }
            Event::MouseUp(button_id) => {
                println!("Mouse up, button id: {:?}", button_id);
            }
            Event::MouseScroll(mouse_scroll) => {
                println!("Mouse scroll, {:?}", mouse_scroll);
            }
            Event::MouseClick(mouse_click) => {
                println!("Mouse click, {:?}", mouse_click);
            }
            Event::KeyDown(keycode) => {
                println!("Key down, keycode: {}", keycode);
            }
            Event::KeyUp(keycode) => {
                println!("Key up, keycode: {}", keycode);
            }
            Event::CharacterInput(char_code) => {
                println!("Character input, char_code: {}", char_code);
            }
            Event::WindowResized(window_info) => {
                println!("Window resized, {:?}", window_info);
            }
            Event::WindowFocus => {
                println!("Window focused");
            }
            Event::WindowUnfocus => {
                println!("Window unfocused");
            }
            Event::WillClose => {
                println!("Window will close");
            }
        }
    }

    fn on_app_message(&mut self, message: Self::AppMessage) {}
}
