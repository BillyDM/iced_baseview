use baseview::{Event, EventStatus, Window, WindowHandler, WindowScalePolicy};
use futures::StreamExt;
use iced_futures::futures;
use iced_futures::futures::channel::mpsc;
use iced_graphics::Viewport;
use iced_native::{command::Action, Debug, Executor, Runtime, Size};
use iced_native::{event::Status, Cache, UserInterface};
use mpsc::SendError;
use raw_window_handle::{HasRawWindowHandle, RawWindowHandle};
use std::cell::RefCell;
use std::mem::ManuallyDrop;
use std::pin::Pin;
use std::rc::Rc;

use crate::application::State;
use crate::{proxy::Proxy, Application, Compositor, Renderer, Settings};

pub(crate) enum RuntimeEvent<Message: 'static + Send> {
    Baseview((baseview::Event, bool)),
    UserEvent(Message),
    MainEventsCleared,
    UpdateSwapChain,
    OnFrame,
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
}

impl<A: Application + 'static + Send> IcedWindow<A> {
    fn new(
        window: &mut baseview::Window<'_>,
        flags: A::Flags,
        scale_policy: WindowScalePolicy,
        logical_width: f64,
        logical_height: f64,
        #[cfg(feature = "with-glow")]
        #[cfg(not(feature = "with-wgpu"))]
        use_max_aa_samples: bool,
        sender: mpsc::UnboundedSender<RuntimeEvent<A::Message>>,
        receiver: mpsc::UnboundedReceiver<RuntimeEvent<A::Message>>,
    ) -> IcedWindow<A> {
        use futures::task;

        #[cfg(feature = "with-wgpu")]
        use iced_graphics::window::Compositor as IGCompositor;

        #[cfg(feature = "with-glow")]
        #[cfg(not(feature = "with-wgpu"))]
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

        let subscription = application.subscription(&mut window_subs);

        for action in init_command.actions() {
            match action {
                Action::Future(future) => {
                    runtime.spawn(future);
                }
                _ => {
                    // TODO
                }
            }
        }

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

        #[cfg(feature = "with-wgpu")]
        let (mut compositor, renderer) =
            <Compositor as IGCompositor>::new(renderer_settings, Some(window))
                .unwrap();

        #[cfg(feature = "with-glow")]
        #[cfg(not(feature = "with-wgpu"))]
        let (context, compositor, renderer) = {
            let mut gl_config = renderer_settings.0;
            let mut iced_config = renderer_settings.1;

            if use_max_aa_samples {
                use crate::renderer::settings::Antialiasing;
                use glow::{Context, HasContext};

                let context = raw_gl_context::GlContext::create(
                    window,
                    raw_gl_context::GlConfig::default(),
                )
                .unwrap();
                context.make_current();

                #[allow(unsafe_code)]
                let max_samples = unsafe {
                    let context = Context::from_loader_function(|s| {
                        context.get_proc_address(s)
                    });

                    context.get_parameter_i32(glow::MAX_SAMPLES)
                };

                context.make_not_current();

                if max_samples >= 2 {
                    gl_config.samples = Some(max_samples as u8);
                }

                iced_config.antialiasing = match max_samples {
                    16 => Some(Antialiasing::MSAAx16),
                    8 => Some(Antialiasing::MSAAx8),
                    4 => Some(Antialiasing::MSAAx4),
                    2 => Some(Antialiasing::MSAAx2),
                    _ => None,
                };
            };

            let context =
                raw_gl_context::GlContext::create(window, gl_config).unwrap();
            context.make_current();

            #[allow(unsafe_code)]
            let (compositor, renderer) = unsafe {
                <Compositor as IGCompositor>::new(iced_config, |s| {
                    context.get_proc_address(s)
                })
                .unwrap()
            };

            context.make_not_current();

            (context, compositor, renderer)
        };

        #[cfg(feature = "with-wgpu")]
        let surface = compositor.create_surface(window);

        let state = State::new(&application, viewport, scale_policy);

        let event_status = Rc::new(RefCell::new(EventStatus::Ignored));

        let (window_queue, window_queue_rx) = WindowQueue::new();

        #[cfg(feature = "with-wgpu")]
        let instance = Box::pin(run_instance(
            application,
            compositor,
            renderer,
            runtime,
            debug,
            receiver,
            window_queue,
            surface,
            state,
            window_subs,
            event_status.clone(),
        ));

        #[cfg(feature = "with-glow")]
        #[cfg(not(feature = "with-wgpu"))]
        let instance = Box::pin(run_instance(
            application,
            compositor,
            renderer,
            runtime,
            debug,
            receiver,
            window_queue,
            context,
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
        }
    }

    /// Open a new child window.
    ///
    /// * `parent` - The parent window.
    /// * `settings` - The settings of the window.
    pub fn open_parented<P>(
        parent: &P,
        settings: Settings<A::Flags>,
    ) -> WindowHandle<A::Message>
    where
        P: HasRawWindowHandle,
    {
        let scale_policy = settings.window.scale;
        let logical_width = settings.window.size.width as f64;
        let logical_height = settings.window.size.height as f64;
        let flags = settings.flags;
        #[cfg(feature = "with-glow")]
        #[cfg(not(feature = "with-wgpu"))]
        let use_max_aa_samples = settings.use_max_aa_samples;

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
                    #[cfg(feature = "with-glow")]
                    #[cfg(not(feature = "with-wgpu"))]
                    use_max_aa_samples,
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
        settings: Settings<A::Flags>,
    ) -> WindowHandle<A::Message> {
        let scale_policy = settings.window.scale;
        let logical_width = settings.window.size.width as f64;
        let logical_height = settings.window.size.height as f64;
        let flags = settings.flags;
        #[cfg(feature = "with-glow")]
        #[cfg(not(feature = "with-wgpu"))]
        let use_max_aa_samples = settings.use_max_aa_samples;

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
                    #[cfg(feature = "with-glow")]
                    #[cfg(not(feature = "with-wgpu"))]
                    use_max_aa_samples,
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
    pub fn open_blocking(settings: Settings<A::Flags>) {
        let scale_policy = settings.window.scale;
        let logical_width = settings.window.size.width as f64;
        let logical_height = settings.window.size.height as f64;
        let flags = settings.flags;
        #[cfg(feature = "with-glow")]
        #[cfg(not(feature = "with-wgpu"))]
        let use_max_aa_samples = settings.use_max_aa_samples;

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
                    #[cfg(feature = "with-glow")]
                    #[cfg(not(feature = "with-wgpu"))]
                    use_max_aa_samples,
                    sender,
                    receiver,
                )
            },
        );
    }
}

impl<A: Application + 'static + Send> WindowHandler for IcedWindow<A> {
    fn on_frame(&mut self, window: &mut Window<'_>) {
        // Send event to render the frame.
        self.sender
            .start_send(RuntimeEvent::UpdateSwapChain)
            .expect("Send event");

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
            .start_send(RuntimeEvent::OnFrame)
            .expect("Send event");

        // Flush all messages. This will block until the instance is finished.
        let _ = self.instance.as_mut().poll(&mut self.runtime_context);

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
        let status = if requests_exit(&event) {
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

        while let Ok(Some(msg)) = self.window_queue_rx.try_next() {
            match msg {
                WindowQueueMessage::CloseWindow => {
                    window.close();
                }
            }
        }

        status
    }
}

// This may appear to be asynchronous, but it is actually a blocking future on the same thread.
// This is a necessary workaround for the issue described here:
// https://github.com/hecrj/iced/pull/597
#[allow(clippy::too_many_arguments)]
async fn run_instance<A, E>(
    mut application: A,
    mut compositor: Compositor,
    mut renderer: Renderer,
    mut runtime: Runtime<E, Proxy<A::Message>, A::Message>,
    mut debug: Debug,
    mut receiver: mpsc::UnboundedReceiver<RuntimeEvent<A::Message>>,
    mut window_queue: WindowQueue,

    #[rustfmt::skip]
    #[cfg(feature = "with-wgpu")]
    mut surface: <Compositor as iced_graphics::window::Compositor>::Surface,

    #[rustfmt::skip]
    #[cfg(feature = "with-glow")]
    #[cfg(not(feature = "with-wgpu"))]
    gl_context: raw_gl_context::GlContext,

    mut state: State<A>,
    mut window_subs: WindowSubs<A::Message>,
    event_status: Rc<RefCell<EventStatus>>,
) where
    A: Application + 'static + Send,
    E: Executor + 'static,
{
    #[cfg(feature = "with-wgpu")]
    use iced_graphics::window::Compositor as IGCompositor;

    #[cfg(feature = "with-glow")]
    #[cfg(not(feature = "with-wgpu"))]
    use iced_graphics::window::GLCompositor as IGCompositor;

    //let clipboard = Clipboard::new(window);  // TODO: clipboard

    let mut viewport_version = state.viewport_version();

    #[cfg(feature = "with-wgpu")]
    {
        let physical_size = state.physical_size();

        compositor.configure_surface(
            &mut surface,
            physical_size.width,
            physical_size.height,
        )
    };

    let mut user_interface = ManuallyDrop::new(build_user_interface(
        &mut application,
        Cache::default(),
        &mut renderer,
        state.logical_size(),
        &mut debug,
    ));

    let mut mouse_interaction =
        user_interface.draw(&mut renderer, state.cursor_position());

    let mut events = Vec::new();
    let mut messages = Vec::new();

    let mut redraw_requested = true;
    let mut did_process_event = false;

    let mut modifiers = iced_core::keyboard::Modifiers::empty();

    debug.startup_finished();

    let mut clipboard = iced_native::clipboard::Null; // TODO: clipboard

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

                let statuses = user_interface.update(
                    &events,
                    state.cursor_position(),
                    &mut renderer,
                    &mut clipboard, // TODO: clipboard
                    &mut messages,
                );

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

                    let statuses = user_interface.update(
                        &events,
                        state.cursor_position(),
                        &mut renderer,
                        &mut clipboard, // TODO: clipboard
                        &mut messages,
                    );

                    debug.event_processing_finished();

                    for event in events.drain(..).zip(statuses.into_iter()) {
                        runtime.broadcast(event);
                    }
                }

                if !messages.is_empty() {
                    let cache =
                        ManuallyDrop::into_inner(user_interface).into_cache();

                    // Update application
                    update(
                        &mut application,
                        &mut runtime,
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
                mouse_interaction =
                    user_interface.draw(&mut renderer, state.cursor_position());
                debug.draw_finished();

                redraw_requested = true;
            }
            RuntimeEvent::UserEvent(message) => {
                messages.push(message);
            }
            RuntimeEvent::UpdateSwapChain => {
                let current_viewport_version = state.viewport_version();

                if viewport_version != current_viewport_version {
                    let physical_size = state.physical_size();

                    #[cfg(feature = "with-wgpu")]
                    compositor.configure_surface(
                        &mut surface,
                        physical_size.width,
                        physical_size.height,
                    );

                    #[cfg(feature = "with-glow")]
                    #[cfg(not(feature = "with-wgpu"))]
                    {
                        gl_context.make_current();
                        compositor.resize_viewport(physical_size);
                        gl_context.make_not_current();
                    }

                    let logical_size = state.logical_size();

                    debug.layout_started();
                    user_interface = ManuallyDrop::new(
                        ManuallyDrop::into_inner(user_interface)
                            .relayout(logical_size, &mut renderer),
                    );
                    debug.layout_finished();

                    debug.draw_started();
                    mouse_interaction = user_interface
                        .draw(&mut renderer, state.cursor_position());
                    debug.draw_finished();

                    viewport_version = current_viewport_version;
                }
            }
            RuntimeEvent::OnFrame => {
                if redraw_requested {
                    debug.render_started();

                    #[cfg(feature = "with-wgpu")]
                    compositor
                        .present(
                            &mut renderer,
                            &mut surface,
                            state.viewport(),
                            state.background_color(),
                            // &primitive,
                            &debug.overlay(),
                        )
                        .unwrap();

                    #[cfg(feature = "with-glow")]
                    #[cfg(not(feature = "with-wgpu"))]
                    {
                        gl_context.make_current();

                        compositor.present(
                            &mut renderer,
                            state.viewport(),
                            state.background_color(),
                            &debug.overlay(),
                        );

                        gl_context.swap_buffers();
                        gl_context.make_not_current();
                    };

                    debug.render_finished();

                    // TODO: set window cursor icon
                    /*
                    window.set_cursor_icon(conversion::mouse_interaction(
                        new_mouse_interaction,
                    ));
                    */

                    redraw_requested = false;

                    // TODO: Handle animations!
                    // Maybe we can use `ControlFlow::WaitUntil` for this.
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
    drop(ManuallyDrop::into_inner(user_interface));
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
    cache: Cache,
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

        for action in command.actions() {
            match action {
                Action::Future(future) => {
                    runtime.spawn(future);
                }
                _ => {
                    // TODO
                }
            }
        }
    }

    let subscription = application.subscription(window_subs);
    runtime.track(subscription);
}
