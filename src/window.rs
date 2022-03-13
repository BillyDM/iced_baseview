use baseview::{Event, EventStatus, Window, WindowHandler, WindowScalePolicy};
use futures::StreamExt;
use iced_futures::futures;
use iced_futures::futures::channel::mpsc;
use iced_graphics::Viewport;
use iced_native::clipboard::Clipboard as IcedClipboard;
use iced_native::event::Status;
use iced_native::user_interface::{self, UserInterface};
use iced_native::{clipboard, Command, Debug, Executor, Runtime, Size};
use mpsc::SendError;
use raw_window_handle::{HasRawWindowHandle, RawWindowHandle};
use std::cell::RefCell;
use std::mem::ManuallyDrop;
use std::pin::Pin;
use std::rc::Rc;

use crate::application::State;
use crate::clipboard::Clipboard;
use crate::proxy::Proxy;
use crate::{Application, Compositor, Renderer, Settings};

pub(crate) enum RuntimeEvent<Message: 'static + Send> {
    Baseview((baseview::Event, bool)),
    UserEvent(Message),
    MainEventsCleared,
    RedrawRequested,
    WillClose,
}

/// This struct creates subscriptions for common window events.
#[allow(missing_debug_implementations)]
pub struct WindowSubs<Message: Clone> {
    /// The message to send right before each rendering frame.
    pub on_frame: Option<Message>,
    /// The message to send when the window is about to close.
    pub on_window_will_close: Option<Message>,
}

impl<Message: Clone> Default for WindowSubs<Message> {
    fn default() -> Self {
        WindowSubs {
            on_frame: None,
            on_window_will_close: None,
        }
    }
}

#[derive(Debug, Clone, Copy)]
enum WindowQueueMessage {
    CloseWindow,
}

/// Used to request things from the `baseview` window.
#[allow(missing_debug_implementations)]
pub struct WindowQueue {
    tx: mpsc::UnboundedSender<WindowQueueMessage>,
}

impl WindowQueue {
    fn new() -> (Self, mpsc::UnboundedReceiver<WindowQueueMessage>) {
        let (tx, rx) = mpsc::unbounded();

        (Self { tx }, rx)
    }

    /// Quit the current application and close the window.
    pub fn close_window(&mut self) -> Result<(), SendError> {
        self.tx.start_send(WindowQueueMessage::CloseWindow)
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
    pub fn send_baseview_event(
        &mut self,
        event: baseview::Event,
    ) -> Result<(), SendError> {
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

unsafe impl<Message: 'static + Send> HasRawWindowHandle
    for WindowHandle<Message>
{
    fn raw_window_handle(&self) -> RawWindowHandle {
        self.bv_handle.raw_window_handle()
    }
}

/// Handles an iced_baseview application
#[allow(missing_debug_implementations)]
pub struct IcedWindow<A: Application + 'static + Send> {
    sender: mpsc::UnboundedSender<RuntimeEvent<A::Message>>,
    instance: Pin<Box<dyn futures::Future<Output = ()>>>,
    runtime_context: futures::task::Context<'static>,
    runtime_rx: mpsc::UnboundedReceiver<A::Message>,
    window_queue_rx: mpsc::UnboundedReceiver<WindowQueueMessage>,
    event_status: Rc<RefCell<EventStatus>>,

    processed_close_signal: bool,
}

impl<A: Application + 'static + Send> IcedWindow<A> {
    fn new(
        window: &mut baseview::Window<'_>,
        flags: A::Flags,
        scale_policy: WindowScalePolicy,
        logical_width: f64,
        logical_height: f64,
        sender: mpsc::UnboundedSender<RuntimeEvent<A::Message>>,
        receiver: mpsc::UnboundedReceiver<RuntimeEvent<A::Message>>,
    ) -> IcedWindow<A> {
        // All of this garbage is based on:
        // https://github.com/iced-rs/iced/blob/master/winit/src/application.rs

        use futures::task;

        #[cfg(feature = "wgpu")]
        use iced_graphics::window::Compositor as IGCompositor;

        #[cfg(feature = "glow")]
        #[cfg(not(feature = "wgpu"))]
        use iced_graphics::window::GLCompositor as IGCompositor;

        let mut debug = Debug::new();
        debug.startup_started();

        let (runtime_tx, runtime_rx) = mpsc::unbounded::<A::Message>();

        let mut runtime = {
            let proxy = Proxy::new(runtime_tx);
            let executor = <A::Executor as Executor>::new().unwrap();

            Runtime::new(executor, proxy)
        };

        let (application, init_command) = {
            let flags = flags;

            runtime.enter(|| A::new(flags))
        };

        let mut window_subs = WindowSubs::default();

        let mut clipboard = Clipboard::default();
        let subscription = application.subscription(&mut window_subs);

        run_command(init_command, &mut runtime, &mut clipboard);
        runtime.track(subscription);

        // Assume scale for now until there is an event with a new one.
        let scale = match scale_policy {
            WindowScalePolicy::ScaleFactor(scale) => scale,
            WindowScalePolicy::SystemScaleFactor => 1.0,
        };

        let physical_size = Size::new(
            (logical_width * scale) as u32,
            (logical_height * scale) as u32,
        );

        let viewport = Viewport::with_physical_size(physical_size, scale);

        let renderer_settings = A::renderer_settings();

        #[cfg(feature = "wgpu")]
        let (mut compositor, renderer) =
            Compositor::new(renderer_settings, Some(window)).unwrap();

        #[cfg(feature = "glow")]
        #[cfg(not(feature = "wgpu"))]
        let (compositor, renderer) = {
            let context = window
                .gl_context()
                .expect("Window was created without OpenGL support");
            unsafe { context.make_current() };

            let (compositor, renderer) = unsafe {
                Compositor::new(renderer_settings, |s| {
                    context.get_proc_address(s)
                })
                .unwrap()
            };

            unsafe { context.make_not_current() };

            (compositor, renderer)
        };

        #[cfg(feature = "wgpu")]
        let surface = compositor.create_surface(window);

        let state = State::new(&application, viewport, scale_policy);

        let event_status = Rc::new(RefCell::new(EventStatus::Ignored));

        let (window_queue, window_queue_rx) = WindowQueue::new();

        #[cfg(feature = "wgpu")]
        let instance = Box::pin(run_instance(
            application,
            compositor,
            renderer,
            runtime,
            clipboard,
            debug,
            receiver,
            window_queue,
            surface,
            state,
            window_subs,
            event_status.clone(),
        ));

        #[cfg(feature = "glow")]
        #[cfg(not(feature = "wgpu"))]
        let instance = Box::pin(run_instance(
            application,
            compositor,
            renderer,
            runtime,
            clipboard,
            debug,
            receiver,
            window_queue,
            state,
            window_subs,
            event_status.clone(),
        ));

        let runtime_context = task::Context::from_waker(task::noop_waker_ref());

        Self {
            sender,
            instance,
            runtime_context,
            runtime_rx,
            window_queue_rx,
            event_status,

            processed_close_signal: false,
        }
    }

    /// Make sure the OpenGL context settings on the window open flags are consistent with the
    /// renderer configuration.
    fn update_gl_context(settings: &mut Settings<A::Flags>) {
        #[cfg(feature = "glow")]
        #[cfg(not(feature = "wgpu"))]
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

    /// Open a new child window.
    ///
    /// * `parent` - The parent window.
    /// * `settings` - The settings of the window.
    pub fn open_parented<P>(
        parent: &P,
        #[allow(unused_mut)] mut settings: Settings<A::Flags>,
    ) -> WindowHandle<A::Message>
    where
        P: HasRawWindowHandle,
    {
        Self::update_gl_context(&mut settings);

        let scale_policy = settings.window.scale;
        let logical_width = settings.window.size.width as f64;
        let logical_height = settings.window.size.height as f64;
        let flags = settings.flags;

        let (sender, receiver) = mpsc::unbounded();
        let sender_clone = sender.clone();

        let bv_handle = Window::open_parented(
            parent,
            settings.window,
            move |window: &mut baseview::Window<'_>| -> IcedWindow<A> {
                IcedWindow::new(
                    window,
                    flags,
                    scale_policy,
                    logical_width,
                    logical_height,
                    sender_clone,
                    receiver,
                )
            },
        );

        WindowHandle::new(bv_handle, sender)
    }

    /// Open a new window as if it had a parent window.
    ///
    /// * `settings` - The settings of the window.
    pub fn open_as_if_parented(
        #[allow(unused_mut)] mut settings: Settings<A::Flags>,
    ) -> WindowHandle<A::Message> {
        Self::update_gl_context(&mut settings);

        let scale_policy = settings.window.scale;
        let logical_width = settings.window.size.width as f64;
        let logical_height = settings.window.size.height as f64;
        let flags = settings.flags;

        let (sender, receiver) = mpsc::unbounded();
        let sender_clone = sender.clone();

        let bv_handle = Window::open_as_if_parented(
            settings.window,
            move |window: &mut baseview::Window<'_>| -> IcedWindow<A> {
                IcedWindow::new(
                    window,
                    flags,
                    scale_policy,
                    logical_width,
                    logical_height,
                    sender_clone,
                    receiver,
                )
            },
        );

        WindowHandle::new(bv_handle, sender)
    }

    /// Open a new window that blocks the current thread until the window is destroyed.
    ///
    /// * `settings` - The settings of the window.
    pub fn open_blocking(
        #[allow(unused_mut)] mut settings: Settings<A::Flags>,
    ) {
        Self::update_gl_context(&mut settings);

        let scale_policy = settings.window.scale;
        let logical_width = settings.window.size.width as f64;
        let logical_height = settings.window.size.height as f64;
        let flags = settings.flags;

        let (sender, receiver) = mpsc::unbounded();

        Window::open_blocking(
            settings.window,
            move |window: &mut baseview::Window<'_>| -> IcedWindow<A> {
                IcedWindow::new(
                    window,
                    flags,
                    scale_policy,
                    logical_width,
                    logical_height,
                    sender,
                    receiver,
                )
            },
        );
    }
}

impl<A: Application + 'static + Send> WindowHandler for IcedWindow<A> {
    fn on_frame(&mut self, window: &mut Window<'_>) {
        if self.processed_close_signal {
            return;
        }

        #[cfg(feature = "glow")]
        #[cfg(not(feature = "wgpu"))]
        let gl_context = window
            .gl_context()
            .expect("Window was created without OpenGL support");
        #[cfg(feature = "glow")]
        #[cfg(not(feature = "wgpu"))]
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
        #[cfg(not(feature = "wgpu"))]
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

    fn on_event(
        &mut self,
        window: &mut Window<'_>,
        event: Event,
    ) -> EventStatus {
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

/// This may appear to be asynchronous, but it is actually a blocking future on the same thread.
/// This is a necessary workaround for the issue described here:
/// https://github.com/hecrj/iced/pull/597
///
/// All of this garbage is based on:
/// <https://github.com/iced-rs/iced/blob/master/winit/src/application.rs>
#[allow(clippy::too_many_arguments)]
async fn run_instance<A, E>(
    mut application: A,
    mut compositor: Compositor,
    mut renderer: Renderer,
    mut runtime: Runtime<E, Proxy<A::Message>, A::Message>,
    mut clipboard: Clipboard,
    mut debug: Debug,
    mut receiver: mpsc::UnboundedReceiver<RuntimeEvent<A::Message>>,
    mut window_queue: WindowQueue,

    #[rustfmt::skip]
    #[cfg(feature = "wgpu")]
    mut surface: <Compositor as iced_graphics::window::Compositor>::Surface,

    mut state: State<A>,
    mut window_subs: WindowSubs<A::Message>,
    event_status: Rc<RefCell<EventStatus>>,
) where
    A: Application + 'static + Send,
    E: Executor + 'static,
{
    #[cfg(feature = "wgpu")]
    use iced_graphics::window::Compositor as IGCompositor;

    #[cfg(feature = "glow")]
    #[cfg(not(feature = "wgpu"))]
    use iced_graphics::window::GLCompositor as IGCompositor;

    let mut viewport_version = state.viewport_version();

    #[cfg(feature = "wgpu")]
    {
        let physical_size = state.physical_size();
        compositor.configure_surface(
            &mut surface,
            physical_size.width,
            physical_size.height,
        );
    }

    let mut user_interface = ManuallyDrop::new(build_user_interface(
        &mut application,
        user_interface::Cache::default(),
        &mut renderer,
        state.logical_size(),
        &mut debug,
    ));

    let mut mouse_interaction = iced_native::mouse::Interaction::default();
    let mut events = Vec::new();
    let mut messages = Vec::new();

    // Triggered whenever a baseview event gets sent
    let mut redraw_requested = true;
    // May be triggered when processing baseview events, will cause the UI to be updated in the next
    // frame
    let mut needs_update = true;
    let mut did_process_event = false;

    let mut modifiers = iced_core::keyboard::Modifiers::empty();

    debug.startup_finished();

    while let Some(event) = receiver.next().await {
        match event {
            RuntimeEvent::Baseview((event, do_send_status)) => {
                state.update(&event, &mut debug);

                crate::conversion::baseview_to_iced_events(
                    event,
                    &mut events,
                    &mut modifiers,
                );

                if events.is_empty() {
                    if do_send_status {
                        *event_status.borrow_mut() = EventStatus::Ignored;
                    }
                    continue;
                }

                debug.event_processing_started();

                let (interface_state, statuses) = user_interface.update(
                    &events,
                    state.cursor_position(),
                    &mut renderer,
                    &mut clipboard,
                    &mut messages,
                );
                // Will trigger an update when the next frame gets drawn
                needs_update |=
                    matches!(interface_state, user_interface::State::Outdated,);

                if do_send_status {
                    let mut final_status = EventStatus::Ignored;
                    for status in &statuses {
                        if let Status::Captured = status {
                            final_status = EventStatus::Captured;
                            break;
                        }
                    }

                    *event_status.borrow_mut() = final_status;
                }

                debug.event_processing_finished();

                for event in events.drain(..).zip(statuses.into_iter()) {
                    runtime.broadcast(event);
                }

                did_process_event = true;
            }
            RuntimeEvent::MainEventsCleared => {
                if let Some(message) = &window_subs.on_frame {
                    messages.push(message.clone());
                }

                if !did_process_event
                    && events.is_empty()
                    && messages.is_empty()
                {
                    continue;
                }
                did_process_event = false;

                if !events.is_empty() {
                    debug.event_processing_started();

                    let (interface_state, statuses) = user_interface.update(
                        &events,
                        state.cursor_position(),
                        &mut renderer,
                        &mut clipboard,
                        &mut messages,
                    );
                    needs_update |= matches!(
                        interface_state,
                        user_interface::State::Outdated,
                    );

                    debug.event_processing_finished();

                    for event in events.drain(..).zip(statuses.into_iter()) {
                        runtime.broadcast(event);
                    }
                }

                // The user interface update may have pushed a new message onto the stack
                needs_update |= !messages.is_empty();
                if needs_update {
                    needs_update = false;

                    let cache =
                        ManuallyDrop::into_inner(user_interface).into_cache();

                    // Update application
                    update(
                        &mut application,
                        &mut runtime,
                        &mut clipboard,
                        &mut debug,
                        &mut messages,
                        &mut window_subs,
                        &mut window_queue,
                    );

                    // Update window
                    state.synchronize(&application);

                    user_interface = ManuallyDrop::new(build_user_interface(
                        &mut application,
                        cache,
                        &mut renderer,
                        state.logical_size(),
                        &mut debug,
                    ));
                }

                debug.draw_started();
                let new_mouse_interaction =
                    user_interface.draw(&mut renderer, state.cursor_position());
                debug.draw_finished();

                if new_mouse_interaction != mouse_interaction {
                    // TODO: Handle mouse cursor icons
                    // window.set_cursor_icon(conversion::mouse_interaction(
                    //     new_mouse_interaction,
                    // ));

                    mouse_interaction = new_mouse_interaction;
                }

                redraw_requested = true;
            }
            RuntimeEvent::UserEvent(message) => {
                messages.push(message);
            }
            RuntimeEvent::RedrawRequested => {
                // Set whenever a baseview event is triggered
                if !redraw_requested {
                    continue;
                }

                debug.render_started();
                let current_viewport_version = state.viewport_version();

                if viewport_version != current_viewport_version {
                    let physical_size = state.physical_size();
                    let logical_size = state.logical_size();

                    debug.layout_started();
                    user_interface = ManuallyDrop::new(
                        ManuallyDrop::into_inner(user_interface)
                            .relayout(logical_size, &mut renderer),
                    );
                    debug.layout_finished();

                    debug.draw_started();
                    let new_mouse_interaction = user_interface
                        .draw(&mut renderer, state.cursor_position());

                    if new_mouse_interaction != mouse_interaction {
                        // TODO: Handle mouse cursor icons
                        // window.set_cursor_icon(conversion::mouse_interaction(
                        //     new_mouse_interaction,
                        // ));

                        mouse_interaction = new_mouse_interaction;
                    }
                    debug.draw_finished();

                    #[cfg(feature = "wgpu")]
                    compositor.configure_surface(
                        &mut surface,
                        physical_size.width,
                        physical_size.height,
                    );

                    #[cfg(feature = "glow")]
                    #[cfg(not(feature = "wgpu"))]
                    compositor.resize_viewport(physical_size);

                    viewport_version = current_viewport_version;
                }

                #[cfg(feature = "wgpu")]
                match compositor.present(
                    &mut renderer,
                    &mut surface,
                    state.viewport(),
                    state.background_color(),
                    &debug.overlay(),
                ) {
                    Ok(()) => {
                        debug.render_finished();

                        // TODO: Handle animations!
                        // Maybe we can use `ControlFlow::WaitUntil` for this.

                        redraw_requested = false;
                    }
                    Err(error) => match error {
                        // This is an unrecoverable error.
                        iced_graphics::window::SurfaceError::OutOfMemory => {
                            panic!("{:?}", error);
                        }
                        _ => {
                            debug.render_finished();

                            // Try rendering again next frame.
                            redraw_requested = true;
                        }
                    },
                }

                // The buffer swap happens in `IcedWindow::on_frame()` for the glow backend
                #[cfg(feature = "glow")]
                #[cfg(not(feature = "wgpu"))]
                {
                    compositor.present(
                        &mut renderer,
                        state.viewport(),
                        state.background_color(),
                        &debug.overlay(),
                    );

                    redraw_requested = false;
                }
            }
            RuntimeEvent::WillClose => {
                if let Some(message) = &window_subs.on_window_will_close {
                    // Send message to user before exiting the loop.

                    messages.push(message.clone());
                    let cache =
                        ManuallyDrop::into_inner(user_interface).into_cache();

                    // Update application
                    update(
                        &mut application,
                        &mut runtime,
                        &mut clipboard,
                        &mut debug,
                        &mut messages,
                        &mut window_subs,
                        &mut window_queue,
                    );

                    // Update window
                    state.synchronize(&application);

                    user_interface = ManuallyDrop::new(build_user_interface(
                        &mut application,
                        cache,
                        &mut renderer,
                        state.logical_size(),
                        &mut debug,
                    ));
                }

                break;
            }
        }
    }

    receiver.close();

    // Manually drop the user interface
    let _ = ManuallyDrop::into_inner(user_interface);
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

/// Builds a [`UserInterface`] for the provided [`Application`], logging
/// [`struct@Debug`] information accordingly.
pub fn build_user_interface<'a, A: Application + 'static + Send>(
    application: &'a mut A,
    cache: user_interface::Cache,
    renderer: &mut Renderer,
    size: Size,
    debug: &mut Debug,
) -> UserInterface<'a, A::Message, Renderer> {
    debug.view_started();
    let view = application.view();
    debug.view_finished();

    debug.layout_started();
    let user_interface = UserInterface::build(view, size, cache, renderer);
    debug.layout_finished();

    user_interface
}

/// Updates an [`Application`] by feeding it the provided messages, spawning any
/// resulting [`Command`], and tracking its [`Subscription`].
pub fn update<A: Application, E: Executor>(
    application: &mut A,
    runtime: &mut Runtime<E, Proxy<A::Message>, A::Message>,
    clipboard: &mut Clipboard,
    debug: &mut Debug,
    messages: &mut Vec<A::Message>,
    window_subs: &mut WindowSubs<A::Message>,
    window_queue: &mut WindowQueue,
) {
    for message in messages.drain(..) {
        debug.log_message(&message);

        debug.update_started();
        let command =
            runtime.enter(|| application.update(window_queue, message));
        debug.update_finished();

        run_command(command, runtime, clipboard);
    }

    let subscription = application.subscription(window_subs);
    runtime.track(subscription);
}

/// Runs the actions of a [`Command`], potentially yielding a new message.
pub fn run_command<Message: 'static + std::fmt::Debug + Send, E: Executor>(
    command: Command<Message>,
    runtime: &mut Runtime<E, Proxy<Message>, Message>,
    clipboard: &mut Clipboard,
) {
    use iced_native::command;
    use iced_native::window;

    for action in command.actions() {
        match action {
            command::Action::Future(future) => {
                runtime.spawn(future);
            }
            command::Action::Clipboard(action) => match action {
                clipboard::Action::Read(set_clipboard) => {
                    let message = set_clipboard(clipboard.read());

                    // TODO: Is this what you're supposed to do? The winit example sends an event to
                    //       the window which would end up doing the same thing.
                    runtime.spawn(Box::pin(futures::future::ready(message)));
                }
                clipboard::Action::Write(contents) => {
                    clipboard.write(contents);
                }
            },
            command::Action::Window(action) => match action {
                // Resizing and moving baseview windows is currently not supported
                window::Action::Resize {
                    width: _width,
                    height: _height,
                } => {}
                window::Action::Move { x: _x, y: _y } => {}
            },
        }
    }
}
