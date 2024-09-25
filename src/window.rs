use std::{cell::RefCell, pin::Pin, rc::Rc, sync::Arc};

use iced_graphics::Compositor;
pub use iced_runtime::core::window::Id;

use baseview::{Event, EventStatus, Window, WindowHandler, WindowOpenOptions};
use iced_runtime::futures::futures::{
    self,
    channel::mpsc::{self, SendError},
};
use raw_window_handle::{HasRawWindowHandle, RawWindowHandle};

use crate::{
    application::{run, Application, DefaultStyle},
    Renderer, Settings,
};

pub enum RuntimeEvent<Message: 'static + Send> {
    Baseview((baseview::Event, bool)),
    UserEvent(iced_runtime::Action<Message>),
    MainEventsCleared,
    RedrawRequested,
    WillClose,
}

pub(crate) struct IcedWindow<A>
where
    A: Application + Send + 'static,
    // E: Executor + 'static,
    // C: window::Compositor<Renderer = A::Renderer> + 'static,
{
    pub sender: mpsc::UnboundedSender<RuntimeEvent<A::Message>>,
    pub instance: Pin<Box<dyn futures::Future<Output = ()>>>,
    pub runtime_context: futures::task::Context<'static>,
    pub runtime_rx: mpsc::UnboundedReceiver<iced_runtime::Action<A::Message>>,
    pub window_queue_rx: mpsc::UnboundedReceiver<WindowCommand>,
    pub event_status: Rc<RefCell<EventStatus>>,

    pub processed_close_signal: bool,
}

impl<A> IcedWindow<A>
where
    A: Application + Send + 'static,
    <A as Application>::Theme: DefaultStyle,
    <A as Application>::Executor: iced_runtime::futures::Executor + 'static,
    <A as Application>::Flags: std::marker::Send,
{
    /// There's no clone implementation, but this is fine.
    fn clone_window_options(window: &WindowOpenOptions) -> WindowOpenOptions {
        WindowOpenOptions {
            title: window.title.clone(),
            ..*window
        }
    }

    /// Open a new window that blocks the current thread until the window is destroyed.
    ///
    /// * `settings` - The settings of the window.
    pub fn open_blocking<C>(flags: A::Flags, settings: Settings)
    where
        C: Compositor<Renderer = Renderer> + 'static,
    {
        let (sender, receiver) = mpsc::unbounded();

        Window::open_blocking(
            Self::clone_window_options(&settings.window),
            move |window: &mut baseview::Window<'_>| -> IcedWindow<A> {
                run::<A, C>(window, flags, settings, sender, receiver).expect("Launch window")
            },
        );
    }

    /// Open a new child window.
    ///
    /// * `parent` - The parent window.
    /// * `settings` - The settings of the window.
    pub fn open_parented<W, C>(
        parent: &W,
        flags: A::Flags,
        settings: Settings,
    ) -> WindowHandle<A::Message>
    where
        W: HasRawWindowHandle,
        C: Compositor<Renderer = Renderer> + 'static,
    {
        let (sender, receiver) = mpsc::unbounded();
        let sender_clone = sender.clone();

        let bv_handle = Window::open_parented(
            parent,
            Self::clone_window_options(&settings.window),
            move |window: &mut baseview::Window<'_>| -> IcedWindow<A> {
                run::<A, C>(window, flags, settings, sender_clone, receiver).expect("Launch window")
            },
        );

        WindowHandle::new(bv_handle, sender)
    }

    fn drain_window_commands(&mut self, window: &mut Window<'_>) {
        while let Ok(Some(cmd)) = self.window_queue_rx.try_next() {
            match cmd {
                WindowCommand::CloseWindow => {
                    window.close();
                }
                WindowCommand::ResizeWindow(size) => {
                    window.resize(baseview::Size {
                        width: size.width as f64,
                        height: size.height as f64,
                    });
                }
                WindowCommand::Focus => {
                    window.focus();
                }
                WindowCommand::SetCursorIcon(cursor) => {
                    window.set_mouse_cursor(cursor);
                }
            }
        }
    }
}

impl<A> WindowHandler for IcedWindow<A>
where
    A: Application + Send + 'static,
    <A as Application>::Theme: DefaultStyle,
    <A as Application>::Executor: iced_runtime::futures::Executor + 'static,
    <A as Application>::Flags: std::marker::Send,
{
    fn on_frame(&mut self, window: &mut Window<'_>) {
        if self.processed_close_signal {
            return;
        }

        // Flush all messages. This will block until the instance is finished.
        let _ = self.instance.as_mut().poll(&mut self.runtime_context);

        // Poll subscriptions and send the corresponding messages.
        while let Ok(Some(message)) = self.runtime_rx.try_next() {
            self.sender
                .start_send(RuntimeEvent::UserEvent(message))
                .expect("Send event");
        }

        // Send the event to the instance.
        self.sender
            .start_send(RuntimeEvent::MainEventsCleared)
            .expect("Send event");

        // Send event to render the frame.
        self.sender
            .start_send(RuntimeEvent::RedrawRequested)
            .expect("Send event");

        // Flush all messages. This will block until the instance is finished.
        let _ = self.instance.as_mut().poll(&mut self.runtime_context);

        self.drain_window_commands(window);
    }

    fn on_event(&mut self, window: &mut Window<'_>, event: Event) -> EventStatus {
        if self.processed_close_signal {
            return EventStatus::Ignored;
        }

        let status = if requests_exit(&event) {
            self.processed_close_signal = true;

            self.sender
                .start_send(RuntimeEvent::WillClose)
                .expect("Send event");

            // Flush all messages so the application receives the close event. This will block until the instance is finished.
            let _ = self.instance.as_mut().poll(&mut self.runtime_context);

            EventStatus::Ignored
        } else {
            // Send the event to the instance.
            self.sender
                .start_send(RuntimeEvent::Baseview((event, true)))
                .expect("Send event");

            // Flush all messages so the application receives the event. This will block until the instance is finished.
            let _ = self.instance.as_mut().poll(&mut self.runtime_context);

            // TODO: make this Copy
            *self.event_status.borrow()
        };

        if !self.processed_close_signal {
            self.drain_window_commands(window);
        }

        status
    }
}

/// Returns true if the provided event should cause an [`Application`] to
/// exit.
pub fn requests_exit(event: &baseview::Event) -> bool {
    match event {
        baseview::Event::Window(baseview::WindowEvent::WillClose) => true,
        #[cfg(target_os = "macos")]
        baseview::Event::Keyboard(event) => {
            if event.code == keyboard_types::Code::KeyQ
                && event.modifiers == keyboard_types::Modifiers::META
                && event.state == keyboard_types::KeyState::Down
            {
                return true;
            }

            false
        }
        _ => false,
    }
}

/// Use this to send custom events to the iced window.
///
/// Please note this channel is ***not*** realtime-safe and should never be
/// be used to send events from the audio thread. Use a realtime-safe ring
/// buffer instead.
#[allow(missing_debug_implementations)]
pub struct WindowHandle<Message: 'static + Send> {
    bv_handle: baseview::WindowHandle,
    tx: mpsc::UnboundedSender<RuntimeEvent<Message>>,
}

impl<Message: 'static + Send> WindowHandle<Message> {
    pub(crate) fn new(
        bv_handle: baseview::WindowHandle,
        tx: mpsc::UnboundedSender<RuntimeEvent<Message>>,
    ) -> Self {
        Self { bv_handle, tx }
    }

    /// Send a custom `baseview::Event` to the window.
    ///
    /// Please note this channel is ***not*** realtime-safe and should never be
    /// be used to send events from the audio thread. Use a realtime-safe ring
    /// buffer instead.
    pub fn send_baseview_event(&mut self, event: baseview::Event) -> Result<(), SendError> {
        self.tx.start_send(RuntimeEvent::Baseview((event, false)))
    }

    /// Send a custom message to the window.
    ///
    /// Please note this channel is ***not*** realtime-safe and should never be
    /// used to send events from the audio thread. Use a realtime-safe ring
    /// buffer instead.
    pub fn send_message(&mut self, msg: Message) -> Result<(), SendError> {
        self.tx
            .start_send(RuntimeEvent::UserEvent(iced_runtime::Action::Output(msg)))
    }

    /// Signal the window to close.
    pub fn close_window(&mut self) {
        self.bv_handle.close();
    }

    /// Returns `true` if the window is still open, and `false` if the window
    /// was closed/dropped.
    pub fn is_open(&self) -> bool {
        self.bv_handle.is_open()
    }
}

unsafe impl<Message: 'static + Send> HasRawWindowHandle for WindowHandle<Message> {
    fn raw_window_handle(&self) -> RawWindowHandle {
        self.bv_handle.raw_window_handle()
    }
}

#[derive(Debug, Clone)]
pub enum WindowCommand {
    CloseWindow,
    ResizeWindow(crate::core::Size),
    Focus,
    SetCursorIcon(baseview::MouseCursor),
}

/// Used to request things from the `baseview` window.
pub struct WindowQueue {
    tx: mpsc::UnboundedSender<WindowCommand>,
}

impl WindowQueue {
    pub fn new() -> (Self, mpsc::UnboundedReceiver<WindowCommand>) {
        let (tx, rx) = mpsc::unbounded();

        (Self { tx }, rx)
    }

    /// Resize the current application window.
    pub fn resize_window(&mut self, size: crate::core::Size) -> Result<(), SendError> {
        self.tx.start_send(WindowCommand::ResizeWindow(size))
    }

    /// Quit the current application and close the window.
    pub fn close_window(&mut self) -> Result<(), SendError> {
        self.tx.start_send(WindowCommand::CloseWindow)
    }

    /// Request to focus the application window.
    pub fn focus(&mut self) -> Result<(), SendError> {
        self.tx.start_send(WindowCommand::Focus)
    }

    pub fn set_mouse_cursor(&mut self, cursor: baseview::MouseCursor) -> Result<(), SendError> {
        self.tx.start_send(WindowCommand::SetCursorIcon(cursor))
    }
}

/// This struct creates subscriptions for common window events.
#[allow(missing_debug_implementations)]
pub struct WindowSubs<Message> {
    /// The message to send right before each rendering frame.
    pub on_frame: Option<Arc<dyn Fn() -> Option<Message>>>,
    /// The message to send when the window is about to close.
    pub on_window_will_close: Option<Arc<dyn Fn() -> Option<Message>>>,
}

impl<Message> Default for WindowSubs<Message> {
    fn default() -> Self {
        WindowSubs {
            on_frame: None,
            on_window_will_close: None,
        }
    }
}
