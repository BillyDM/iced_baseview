use baseview::{Event, Window, WindowHandler, WindowScalePolicy};
use iced_futures::futures;
use iced_futures::futures::channel::mpsc;
use iced_graphics::Viewport;
use iced_native::{Cache, UserInterface};
use iced_native::{Debug, Executor, Runtime, Size};
use raw_window_handle::{HasRawWindowHandle, RawWindowHandle};
use std::mem::ManuallyDrop;
use std::pin::Pin;

use crate::application::State;
use crate::{proxy::Proxy, Application, Compositor, Renderer, Settings};

enum RuntimeEvent<Message: 'static + Send> {
    Baseview(baseview::Event),
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

/// Handles an iced_baseview application
#[allow(missing_debug_implementations)]
pub struct IcedWindow<A: Application + 'static + Send> {
    sender: mpsc::UnboundedSender<RuntimeEvent<A::Message>>,
    instance: Pin<Box<dyn futures::Future<Output = ()>>>,
    runtime_context: futures::task::Context<'static>,
    runtime_rx: mpsc::UnboundedReceiver<A::Message>,
}

impl<A: Application + 'static + Send> IcedWindow<A> {
    fn new(
        window: &mut baseview::Window<'_>,
        flags: A::Flags,
        scale_policy: WindowScalePolicy,
        logical_width: f64,
        logical_height: f64,
    ) -> IcedWindow<A> {
        use futures::task;
        use iced_graphics::window::Compositor as IGCompositor;

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

        runtime.spawn(init_command);
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

        let (mut compositor, renderer) =
            <Compositor as IGCompositor>::new(renderer_settings).unwrap();

        let surface = compositor.create_surface(window);

        let state = State::new(&application, viewport, scale_policy);

        let (sender, receiver) = mpsc::unbounded();

        let instance = Box::pin(run_instance(
            application,
            compositor,
            renderer,
            runtime,
            debug,
            receiver,
            surface,
            state,
            window_subs,
        ));

        let runtime_context = task::Context::from_waker(task::noop_waker_ref());

        Self {
            sender,
            instance,
            runtime_context,
            runtime_rx,
        }
    }

    /// Open a new child window.
    ///
    /// * `parent` - The parent window.
    /// * `settings` - The settings of the window.
    pub fn open_parented<P>(parent: &P, settings: Settings<A::Flags>)
    where
        P: HasRawWindowHandle,
    {
        // WindowScalePolicy does not implement Copy/Clone.
        let scale_policy = match &settings.window.scale {
            WindowScalePolicy::SystemScaleFactor => {
                WindowScalePolicy::SystemScaleFactor
            }
            WindowScalePolicy::ScaleFactor(scale) => {
                WindowScalePolicy::ScaleFactor(*scale)
            }
        };
        let logical_width = settings.window.size.width as f64;
        let logical_height = settings.window.size.height as f64;
        let flags = settings.flags;

        Window::open_parented(
            parent,
            settings.window,
            move |window: &mut baseview::Window<'_>| -> IcedWindow<A> {
                IcedWindow::new(
                    window,
                    flags,
                    scale_policy,
                    logical_width,
                    logical_height,
                )
            },
        )
    }

    /// Open a new window as if it had a parent window.
    ///
    /// * `settings` - The settings of the window.
    pub fn open_as_if_parented(
        settings: Settings<A::Flags>,
    ) -> RawWindowHandle {
        // WindowScalePolicy does not implement Copy/Clone.
        let scale_policy = match &settings.window.scale {
            WindowScalePolicy::SystemScaleFactor => {
                WindowScalePolicy::SystemScaleFactor
            }
            WindowScalePolicy::ScaleFactor(scale) => {
                WindowScalePolicy::ScaleFactor(*scale)
            }
        };
        let logical_width = settings.window.size.width as f64;
        let logical_height = settings.window.size.height as f64;
        let flags = settings.flags;

        Window::open_as_if_parented(
            settings.window,
            move |window: &mut baseview::Window<'_>| -> IcedWindow<A> {
                IcedWindow::new(
                    window,
                    flags,
                    scale_policy,
                    logical_width,
                    logical_height,
                )
            },
        )
    }

    /// Open a new window that blocks the current thread until the window is destroyed.
    ///
    /// * `settings` - The settings of the window.
    pub fn open_blocking(settings: Settings<A::Flags>) {
        // WindowScalePolicy does not implement Copy/Clone.
        let scale_policy = match &settings.window.scale {
            WindowScalePolicy::SystemScaleFactor => {
                WindowScalePolicy::SystemScaleFactor
            }
            WindowScalePolicy::ScaleFactor(scale) => {
                WindowScalePolicy::ScaleFactor(*scale)
            }
        };
        let logical_width = settings.window.size.width as f64;
        let logical_height = settings.window.size.height as f64;
        let flags = settings.flags;

        Window::open_blocking(
            settings.window,
            move |window: &mut baseview::Window<'_>| -> IcedWindow<A> {
                IcedWindow::new(
                    window,
                    flags,
                    scale_policy,
                    logical_width,
                    logical_height,
                )
            },
        )
    }
}

impl<A: Application + 'static + Send> WindowHandler for IcedWindow<A> {
    fn on_frame(&mut self, _window: &mut Window<'_>) {
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
    }

    fn on_event(&mut self, _window: &mut Window<'_>, event: Event) {
        if requests_exit(&event) {
            self.sender
                .start_send(RuntimeEvent::WillClose)
                .expect("Send event");

            // Flush all messages so the application receives the close event. This will block until the instance is finished.
            let _ = self.instance.as_mut().poll(&mut self.runtime_context);
        } else {
            // Send the event to the instance.
            self.sender
                .start_send(RuntimeEvent::Baseview(event))
                .expect("Send event");
        }
    }
}

// This may appear to be asynchronous, but it is actually a blocking future on the same thread.
// This is a necessary workaround for the issue described here:
// https://github.com/hecrj/iced/pull/597
async fn run_instance<A, E>(
    mut application: A,
    mut compositor: Compositor,
    mut renderer: Renderer,
    mut runtime: Runtime<E, Proxy<A::Message>, A::Message>,
    mut debug: Debug,
    mut receiver: mpsc::UnboundedReceiver<RuntimeEvent<A::Message>>,
    surface: <Compositor as iced_graphics::window::Compositor>::Surface,
    mut state: State<A>,
    mut window_subs: WindowSubs<A::Message>,
) where
    A: Application + 'static + Send,
    E: Executor + 'static,
{
    use iced_futures::futures::stream::StreamExt;
    use iced_graphics::window::Compositor as IGCompositor;
    //let clipboard = Clipboard::new(window);  // TODO: clipboard

    let mut viewport_version = state.viewport_version();
    let mut swap_chain = {
        let physical_size = state.physical_size();

        compositor.create_swap_chain(
            &surface,
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

    let mut primitive =
        user_interface.draw(&mut renderer, state.cursor_position());
    let mut mouse_interaction = iced_native::mouse::Interaction::default();

    let mut events = Vec::new();
    let mut messages = Vec::new();

    let mut redraw_requested = true;

    let mut modifiers = iced_core::keyboard::Modifiers {
        shift: false,
        control: false,
        alt: false,
        logo: false,
    };

    debug.startup_finished();

    while let Some(event) = receiver.next().await {
        match event {
            RuntimeEvent::Baseview(event) => {
                state.update(&event, &mut debug);

                crate::conversion::baseview_to_iced_events(
                    event,
                    &mut events,
                    &mut modifiers,
                );
            }
            RuntimeEvent::MainEventsCleared => {
                if let Some(message) = &window_subs.on_frame {
                    messages.push(message.clone());
                }

                if events.is_empty() && messages.is_empty() {
                    continue;
                }

                debug.event_processing_started();

                let statuses = user_interface.update(
                    &events,
                    state.cursor_position(),
                    None, // TODO: clipboard
                    &mut renderer,
                    &mut messages,
                );

                debug.event_processing_finished();

                for event in events.drain(..).zip(statuses.into_iter()) {
                    runtime.broadcast(event);
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
                primitive =
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

                    swap_chain = compositor.create_swap_chain(
                        &surface,
                        physical_size.width,
                        physical_size.height,
                    );

                    let logical_size = state.logical_size();

                    debug.layout_started();
                    user_interface = ManuallyDrop::new(
                        ManuallyDrop::into_inner(user_interface)
                            .relayout(logical_size, &mut renderer),
                    );
                    debug.layout_finished();

                    debug.draw_started();
                    primitive = user_interface
                        .draw(&mut renderer, state.cursor_position());
                    debug.draw_finished();

                    viewport_version = current_viewport_version;
                }
            }
            RuntimeEvent::OnFrame => {
                if redraw_requested {
                    debug.render_started();

                    let new_mouse_interaction = compositor.draw(
                        &mut renderer,
                        &mut swap_chain,
                        state.viewport(),
                        state.background_color(),
                        &primitive,
                        &debug.overlay(),
                    );

                    debug.render_finished();

                    if new_mouse_interaction != mouse_interaction {
                        // TODO: set window cursor icon
                        /*
                        window.set_cursor_icon(conversion::mouse_interaction(
                            new_mouse_interaction,
                        ));
                        */

                        mouse_interaction = new_mouse_interaction;
                    }

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
            if event.code == keyboard_types::Code::KeyQ {
                if event.modifiers == keyboard_types::Modifiers::META {
                    if event.state == keyboard_types::KeyState::Down {
                        return true;
                    }
                }
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
) {
    for message in messages.drain(..) {
        debug.log_message(&message);

        debug.update_started();
        let command = runtime.enter(|| application.update(message));
        debug.update_finished();

        runtime.spawn(command);
    }

    let subscription = application.subscription(window_subs);
    runtime.track(subscription);
}
