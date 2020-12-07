use baseview::{Event, Window, WindowHandler};
use iced_futures::futures;
use iced_futures::futures::channel::mpsc;
use iced_graphics::Viewport;
use iced_native::{Cache, UserInterface};
use iced_native::{Debug, Executor, Runtime, Size};
use std::mem::ManuallyDrop;
use std::pin::Pin;

use crate::application::State;
use crate::{
    proxy::Proxy, Application, Compositor, Parent, Renderer, Settings,
    WindowScalePolicy,
};

enum RuntimeEvent<Message: 'static + Send> {
    Baseview(baseview::Event),
    UserEvent(Message),
    MainEventsCleared,
    OnFrame,
}

/// Handles an iced_baseview application
#[allow(missing_debug_implementations)]
pub struct Runner<A: Application + 'static + Send> {
    sender: mpsc::UnboundedSender<RuntimeEvent<A::Message>>,
    instance: Pin<Box<dyn futures::Future<Output = ()>>>,
    runtime_context: futures::task::Context<'static>,
    runtime_rx: mpsc::UnboundedReceiver<A::Message>,
}

impl<A: Application + 'static + Send> Runner<A> {
    /// Open a new window
    pub fn open(
        settings: Settings<A::Flags>,
        parent: Parent,
    ) -> (baseview::WindowHandle<Self>, Option<baseview::AppRunner>) {
        let scale_policy = settings.window.scale_policy;

        let logical_width = settings.window.logical_size.0 as f64;
        let logical_height = settings.window.logical_size.1 as f64;

        let window_settings = baseview::WindowOpenOptions {
            title: String::from("test"),
            size: baseview::Size::new(logical_width, logical_height),
            scale: settings.window.scale_policy.into(),
            parent,
        };

        Window::open(
            window_settings,
            move |window: &mut baseview::Window<'_>| -> Runner<A> {
                use iced_graphics::window::Compositor as IGCompositor;

                use futures::task;

                let mut debug = Debug::new();
                debug.startup_started();

                let (runtime_tx, runtime_rx) = mpsc::unbounded::<A::Message>();

                let mut runtime = {
                    let proxy = Proxy::new(runtime_tx);
                    let executor = <A::Executor as Executor>::new().unwrap();

                    Runtime::new(executor, proxy)
                };

                let (application, init_command) = {
                    let flags = settings.flags;

                    runtime.enter(|| A::new(flags))
                };

                let subscription = application.subscription();

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

                let viewport =
                    Viewport::with_physical_size(physical_size, scale);

                let renderer_settings = A::renderer_settings();

                let (mut compositor, renderer) =
                    <Compositor as IGCompositor>::new(renderer_settings)
                        .unwrap();

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
                ));

                let runtime_context =
                    task::Context::from_waker(task::noop_waker_ref());

                Self {
                    sender,
                    instance,
                    runtime_context,
                    runtime_rx,
                }
            },
        )
    }
}

impl<A: Application + 'static + Send> WindowHandler for Runner<A> {
    // TODO: Add message API
    type Message = ();

    fn on_frame(&mut self) {
        loop {
            if let Ok(Some(message)) = self.runtime_rx.try_next() {
                self.sender
                    .start_send(RuntimeEvent::UserEvent(message))
                    .expect("Send event");
            } else {
                break;
            }
        }

        self.sender
            .start_send(RuntimeEvent::MainEventsCleared)
            .expect("Send event");

        self.sender
            .start_send(RuntimeEvent::OnFrame)
            .expect("Send event");

        let _ = self.instance.as_mut().poll(&mut self.runtime_context);
    }

    fn on_event(&mut self, _window: &mut Window<'_>, event: Event) {
        self.sender
            .start_send(RuntimeEvent::Baseview(event))
            .expect("Send event");
    }

    fn on_message(
        &mut self,
        _window: &mut Window<'_>,
        _message: Self::Message,
    ) {
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

    debug.startup_finished();

    while let Some(event) = receiver.next().await {
        match event {
            RuntimeEvent::Baseview(event) => {
                if requests_exit(&event) {
                    break;
                }

                state.update(&event, &mut debug);

                if let Some(iced_event) =
                    crate::conversion::baseview_to_iced_event(event)
                {
                    events.push(iced_event);
                }
            }
            RuntimeEvent::MainEventsCleared => {
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
            RuntimeEvent::OnFrame => {
                if redraw_requested {
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
                        primitive = user_interface
                            .draw(&mut renderer, state.cursor_position());
                        debug.draw_finished();

                        swap_chain = compositor.create_swap_chain(
                            &surface,
                            physical_size.width,
                            physical_size.height,
                        );

                        viewport_version = current_viewport_version;
                    }

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
) {
    for message in messages.drain(..) {
        debug.log_message(&message);

        debug.update_started();
        let command = runtime.enter(|| application.update(message));
        debug.update_finished();

        runtime.spawn(command);
    }

    let subscription = application.subscription();
    runtime.track(subscription);
}
