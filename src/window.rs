use std::{cell::RefCell, pin::Pin, rc::Rc};

use baseview::{Event, EventStatus, Window, WindowHandler, WindowOpenOptions};
use iced_futures::futures::{
    self,
    channel::mpsc::{self, SendError},
};
use iced_native::application::StyleSheet;
use raw_window_handle::{HasRawWindowHandle, RawWindowHandle};

use crate::{application::run, application::Application, Settings};

pub(crate) enum RuntimeEvent<Message: 'static + Send> {
    Baseview((baseview::Event, bool)),
    UserEvent(Message),
    MainEventsCleared,
    RedrawRequested,
    WillClose,
}

pub(crate) struct IcedWindow<A>
where
    A: Application + Send + 'static,
    <A::Renderer as iced_native::Renderer>::Theme: StyleSheet,
    // E: Executor + 'static,
    // C: window::Compositor<Renderer = A::Renderer> + 'static,
{
    pub sender: mpsc::UnboundedSender<RuntimeEvent<A::Message>>,
    pub instance: Pin<Box<dyn futures::Future<Output = ()>>>,
    pub runtime_context: futures::task::Context<'static>,
    pub runtime_rx: mpsc::UnboundedReceiver<A::Message>,
    pub window_queue_rx: mpsc::UnboundedReceiver<WindowQueueMessage>,
    pub event_status: Rc<RefCell<EventStatus>>,

    pub processed_close_signal: bool,
}

impl<A> IcedWindow<A>
where
    A: Application + Send + 'static,
    <A::Renderer as iced_native::Renderer>::Theme: StyleSheet,
{
    /// There's no clone implementation, but this is fine.
    fn clone_window_options(window: &WindowOpenOptions) -> WindowOpenOptions {
        WindowOpenOptions {
            title: window.title.clone(),
            #[cfg(feature = "glow")]
            gl_config: window.gl_config.clone(),
            ..*window
        }
    }

    /// Make sure the OpenGL context settings on the window open flags are consistent with the
    /// renderer configuration.
    #[cfg(feature = "glow")]
    fn update_gl_context(settings: &mut Settings<A::Flags>) {
        {
            // Glow support requires, well, OpenGL
            let gl_config = settings
                .window
                .gl_config
                // FIXME: The current glow_glpyh version does not enable the correct extension in their
                //        shader so this currently won't work with OpenGL <= 3.2
                .get_or_insert_with(|| baseview::gl::GlConfig {
                    version: (3, 3),
                    ..baseview::gl::GlConfig::default()
                });

            // Make sure the anti aliasing settings match up if they have been set on renderer's
            // settings
            if let Some(antialiasing) = A::renderer_settings().antialiasing {
                gl_config.samples = Some(antialiasing.sample_count() as u8);
            }
        }
    }

    /// Open a new window that blocks the current thread until the window is destroyed.
    ///
    /// * `settings` - The settings of the window.
    pub fn open_blocking<E, C>(#[allow(unused_mut)] mut settings: Settings<A::Flags>)
    where
        E: iced_futures::Executor + 'static,
        C: crate::IGCompositor<Renderer = A::Renderer, Settings = crate::renderer::Settings>
            + 'static,
        <C as crate::IGCompositor>::Settings: Send,
    {
        #[cfg(feature = "glow")]
        Self::update_gl_context(&mut settings);

        let (sender, receiver) = mpsc::unbounded();

        Window::open_blocking(
            Self::clone_window_options(&settings.window),
            move |window: &mut baseview::Window<'_>| -> IcedWindow<A> {
                run::<A, E, C>(window, settings, sender, receiver).expect("Launch window")
            },
        );
    }

    /// Open a new child window.
    ///
    /// * `parent` - The parent window.
    /// * `settings` - The settings of the window.
    pub fn open_parented<E, C, P>(
        parent: &P,
        #[allow(unused_mut)] mut settings: Settings<A::Flags>,
    ) -> WindowHandle<A::Message>
    where
        E: iced_futures::Executor + 'static,
        C: crate::IGCompositor<Renderer = A::Renderer, Settings = crate::renderer::Settings>
            + 'static,
        <C as crate::IGCompositor>::Settings: Send,
        P: HasRawWindowHandle,
    {
        #[cfg(feature = "glow")]
        Self::update_gl_context(&mut settings);

        let (sender, receiver) = mpsc::unbounded();
        let sender_clone = sender.clone();

        let bv_handle = Window::open_parented(
            parent,
            Self::clone_window_options(&settings.window),
            move |window: &mut baseview::Window<'_>| -> IcedWindow<A> {
                run::<A, E, C>(window, settings, sender_clone, receiver).expect("Launch window")
            },
        );

        WindowHandle::new(bv_handle, sender)
    }

    /// Open a new window as if it had a parent window.
    ///
    /// * `settings` - The settings of the window.
    pub fn open_as_if_parented<E, C>(
        #[allow(unused_mut)] mut settings: Settings<A::Flags>,
    ) -> WindowHandle<A::Message>
    where
        E: iced_futures::Executor + 'static,
        C: crate::IGCompositor<Renderer = A::Renderer, Settings = crate::renderer::Settings>
            + 'static,
        <C as crate::IGCompositor>::Settings: Send,
    {
        #[cfg(feature = "glow")]
        Self::update_gl_context(&mut settings);

        let (sender, receiver) = mpsc::unbounded();
        let sender_clone = sender.clone();

        let bv_handle = Window::open_as_if_parented(
            Self::clone_window_options(&settings.window),
            move |window: &mut baseview::Window<'_>| -> IcedWindow<A> {
                run::<A, E, C>(window, settings, sender_clone, receiver).expect("Launch window")
            },
        );

        WindowHandle::new(bv_handle, sender)
    }
}

impl<A> WindowHandler for IcedWindow<A>
where
    A: Application + Send + 'static,
    <A::Renderer as iced_native::Renderer>::Theme: StyleSheet,
{
    fn on_frame(&mut self, window: &mut Window<'_>) {
        if self.processed_close_signal {
            return;
        }

        #[cfg(feature = "glow")]
        let gl_context = window
            .gl_context()
            .expect("Window was created without OpenGL support");
        #[cfg(feature = "glow")]
        unsafe {
            gl_context.make_current()
        };

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

        // FIXME: We can't do this inside of the `run_instance()` future. That should probably be
        //        replaced entirely.
        #[cfg(feature = "glow")]
        {
            gl_context.swap_buffers();
            unsafe { gl_context.make_not_current() };
        }

        while let Ok(Some(msg)) = self.window_queue_rx.try_next() {
            match msg {
                WindowQueueMessage::CloseWindow => {
                    window.close();
                }
            }
        }
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
            while let Ok(Some(msg)) = self.window_queue_rx.try_next() {
                match msg {
                    WindowQueueMessage::CloseWindow => {
                        window.close();
                    }
                }
            }
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
        self.tx.start_send(RuntimeEvent::UserEvent(msg))
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

#[derive(Debug, Clone, Copy)]
pub enum WindowQueueMessage {
    CloseWindow,
}

/// Used to request things from the `baseview` window.
#[allow(missing_debug_implementations)]
pub struct WindowQueue {
    tx: mpsc::UnboundedSender<WindowQueueMessage>,
}

impl WindowQueue {
    pub fn new() -> (Self, mpsc::UnboundedReceiver<WindowQueueMessage>) {
        let (tx, rx) = mpsc::unbounded();

        (Self { tx }, rx)
    }

    /// Quit the current application and close the window.
    pub fn close_window(&mut self) -> Result<(), SendError> {
        self.tx.start_send(WindowQueueMessage::CloseWindow)
    }
}

/// This struct creates subscriptions for common window events.
#[allow(missing_debug_implementations)]
pub struct WindowSubs<Message> {
    /// The message to send right before each rendering frame.
    pub on_frame: Option<fn() -> Message>,
    /// The message to send when the window is about to close.
    pub on_window_will_close: Option<fn() -> Message>,
}

impl<Message> Default for WindowSubs<Message> {
    fn default() -> Self {
        WindowSubs {
            on_frame: None,
            on_window_will_close: None,
        }
    }
}
