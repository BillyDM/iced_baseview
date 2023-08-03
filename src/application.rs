//! Create interactive, native cross-platform applications.
#[cfg(feature = "trace")]
mod profiler;
mod state;

use baseview::EventStatus;
pub use state::State;

use crate::core;
use crate::core::mouse;
use crate::core::renderer;
use crate::core::widget::operation;
use crate::core::Size;
use crate::futures::futures;
use crate::futures::{Executor, Runtime, Subscription};
use crate::graphics::compositor::{self, Compositor};
use crate::runtime::clipboard;
use crate::runtime::program::Program;
use crate::runtime::user_interface::{self, UserInterface};
use crate::runtime::{Command, Debug};
use crate::style::application::{Appearance, StyleSheet};
use crate::window::{IcedWindow, RuntimeEvent, WindowQueue, WindowSubs};
use crate::{Clipboard, Error, Proxy, Settings};

use futures::channel::mpsc;

use std::cell::RefCell;
use std::mem::ManuallyDrop;
use std::rc::Rc;

#[cfg(feature = "trace")]
pub use profiler::Profiler;
#[cfg(feature = "trace")]
use tracing::{info_span, instrument::Instrument};

/// An interactive, native cross-platform application.
///
/// This trait is the main entrypoint of Iced. Once implemented, you can run
/// your GUI application by simply calling [`run`]. It will run in
/// its own window.
///
/// An [`Application`] can execute asynchronous actions by returning a
/// [`Command`] in some of its methods.
///
/// When using an [`Application`] with the `debug` feature enabled, a debug view
/// can be toggled by pressing `F12`.
pub trait Application: Program
where
    <Self::Renderer as core::Renderer>::Theme: StyleSheet,
{
    /// The data needed to initialize your [`Application`].
    type Flags;

    /// Initializes the [`Application`] with the flags provided to
    /// [`run`] as part of the [`Settings`].
    ///
    /// Here is where you should return the initial state of your app.
    ///
    /// Additionally, you can return a [`Command`] if you need to perform some
    /// async action in the background on startup. This is useful if you want to
    /// load state from a file, perform an initial HTTP request, etc.
    fn new(flags: Self::Flags) -> (Self, Command<Self::Message>);

    /// Returns the current title of the [`Application`].
    ///
    /// This title can be dynamic! The runtime will automatically update the
    /// title of your application when necessary.
    fn title(&self) -> String;

    /// Returns the current `Theme` of the [`Application`].
    fn theme(&self) -> <Self::Renderer as core::Renderer>::Theme;

    /// Returns the `Style` variation of the `Theme`.
    fn style(
        &self,
    ) -> <<Self::Renderer as core::Renderer>::Theme as StyleSheet>::Style {
        Default::default()
    }

    /// Returns the event `Subscription` for the current state of the
    /// application.
    ///
    /// The messages produced by the `Subscription` will be handled by
    /// [`update`](#tymethod.update).
    ///
    /// A `Subscription` will be kept alive as long as you keep returning it!
    ///
    /// By default, it returns an empty subscription.
    fn subscription(
        &self,
        _window_subs: &mut WindowSubs<Self::Message>,
    ) -> Subscription<Self::Message> {
        Subscription::none()
    }

    /// Ignore non-modifier keyboard keys. Overrides the field in
    /// `IcedBaseviewSettings` if set
    fn ignore_non_modifier_keys(&self) -> Option<bool> {
        None
    }

    /// Returns the [`WindowScalePolicy`] that the [`Application`] should use.
    ///
    /// By default, it returns `WindowScalePolicy::SystemScaleFactor`.
    ///
    /// [`WindowScalePolicy`]: ../settings/enum.WindowScalePolicy.html
    /// [`Application`]: trait.Application.html
    fn scale_policy(&self) -> baseview::WindowScalePolicy {
        baseview::WindowScalePolicy::SystemScaleFactor
    }
}

/// Runs an [`Application`] with an executor, compositor, and the provided
/// settings.
pub fn run<A, E, C>(
    window: &mut baseview::Window<'_>,
    settings: Settings<A::Flags>,
    // compositor_settings: C::Settings,
    event_sender: mpsc::UnboundedSender<RuntimeEvent<A::Message>>,
    event_receiver: mpsc::UnboundedReceiver<RuntimeEvent<A::Message>>,
) -> Result<IcedWindow<A>, Error>
where
    A: Application + 'static + Send,
    E: Executor + 'static,
    C: Compositor<Renderer = A::Renderer> + 'static,
    <A::Renderer as core::Renderer>::Theme: StyleSheet,
{
    use futures::task;

    #[cfg(feature = "trace")]
    let _guard = Profiler::init();

    let mut debug = Debug::new();
    debug.startup_started();

    #[cfg(feature = "trace")]
    let _ = info_span!("Application", "RUN").entered();

    let viewport = {
        // Assume scale for now until there is an event with a new one.
        let scale = match settings.window.scale {
            baseview::WindowScalePolicy::ScaleFactor(scale) => scale,
            baseview::WindowScalePolicy::SystemScaleFactor => 1.0,
        };

        let physical_size = Size::new(
            (settings.window.size.width * scale) as u32,
            (settings.window.size.height * scale) as u32,
        );

        iced_graphics::Viewport::with_physical_size(physical_size, scale)
    };

    let (runtime_tx, runtime_rx) = mpsc::unbounded::<A::Message>();

    let runtime = {
        let proxy = Proxy::new(runtime_tx);
        let executor = E::new().map_err(Error::ExecutorCreationFailed)?;

        Runtime::new(executor, proxy)
    };

    let (application, init_command) = {
        let flags = settings.flags;

        runtime.enter(|| A::new(flags))
    };

    let compositor_settings = Default::default();

    let window = crate::wrapper::WindowHandleWrapper(window);
    let (mut compositor, renderer) =
        C::new(compositor_settings, Some(&window))?;
    let surface = compositor.create_surface(&window, viewport.physical_width(), viewport.physical_height());

    let (window_queue, window_queue_rx) = WindowQueue::new();
    let event_status = Rc::new(RefCell::new(baseview::EventStatus::Ignored));

    let state = State::new(&application, viewport);

    let instance = Box::pin({
        let run_instance = run_instance::<A, E, C>(
            application,
            compositor,
            renderer,
            runtime,
            debug,
            event_receiver,
            init_command,

            settings.iced_baseview,
            surface,
            event_status.clone(),
            state,
            window_queue,
        );

        #[cfg(feature = "trace")]
        let run_instance =
            run_instance.instrument(info_span!("Application", "LOOP"));

        run_instance
    });

    let runtime_context = task::Context::from_waker(task::noop_waker_ref());

    Ok(IcedWindow {
        sender: event_sender,
        instance,
        runtime_context,
        runtime_rx,
        window_queue_rx,
        event_status,

        processed_close_signal: false,
    })
}

async fn run_instance<A, E, C>(
    mut application: A,
    mut compositor: C,
    mut renderer: A::Renderer,
    mut runtime: Runtime<E, Proxy<A::Message>, A::Message>,
    mut debug: Debug,
    mut event_receiver: mpsc::UnboundedReceiver<RuntimeEvent<A::Message>>,
    init_command: Command<A::Message>,

    settings: crate::settings::IcedBaseviewSettings,
    mut surface: C::Surface,
    event_status: Rc<RefCell<baseview::EventStatus>>,
    mut state: State<A>,
    mut window_queue: WindowQueue,
) where
    A: Application + 'static,
    E: Executor + 'static,
    C: Compositor<Renderer = A::Renderer> + 'static,
    <A::Renderer as core::Renderer>::Theme: StyleSheet,
{
    use futures::stream::StreamExt;

    let mut viewport_version = state.viewport_version();

    let mut clipboard = Clipboard::new();
    let mut cache = user_interface::Cache::default();
    let mut window_subs = WindowSubs::default();

    run_command(
        &application,
        &mut cache,
        &state,
        &mut renderer,
        init_command,
        &mut runtime,
        &mut clipboard,
        &mut debug,
        &mut window_queue,
    );
    runtime.track(application.subscription(&mut window_subs).into_recipes());

    let mut user_interface = ManuallyDrop::new(build_user_interface(
        &application,
        cache,
        &mut renderer,
        state.logical_size(),
        &mut debug,
    ));

    let mut mouse_interaction = mouse::Interaction::default();
    let mut events = Vec::new();
    let mut messages = Vec::new();

    // Triggered whenever a baseview event gets sent
    let mut redraw_requested = true;
    // May be triggered when processing baseview events, will cause the UI to be updated in the next
    // frame
    let mut needs_update = true;
    let mut did_process_event = false;

    debug.startup_finished();

    while let Some(event) = event_receiver.next().await {
        match event {
            RuntimeEvent::MainEventsCleared => {
                if let Some(message) = &window_subs.on_frame {
                    messages.push(message());
                }

                if !did_process_event
                    && events.is_empty()
                    && messages.is_empty()
                    && !settings.always_redraw
                {
                    continue;
                }
                did_process_event = false;

                if !events.is_empty() {
                    debug.event_processing_started();

                    let (interface_state, statuses) = user_interface.update(
                        &events,
                        state.cursor(),
                        &mut renderer,
                        &mut clipboard,
                        &mut messages,
                    );

                    needs_update |= matches!(interface_state, user_interface::State::Outdated,);

                    debug.event_processing_finished();

                    for (event, status) in events.drain(..).zip(statuses.into_iter()) {
                        runtime.broadcast(event, status);
                    }
                }

                // The user interface update may have pushed a new message onto the stack
                needs_update |= !messages.is_empty() || settings.always_redraw;

                if needs_update {
                    needs_update = false;

                    let mut cache = ManuallyDrop::into_inner(user_interface).into_cache();

                    // Update application
                    update(
                        &mut application,
                        &mut cache,
                        &state,
                        &mut renderer,
                        &mut runtime,
                        &mut clipboard,
                        &mut debug,
                        &mut messages,
                        &mut window_subs,
                        &mut window_queue,
                    );

                    // Update window
                    state.synchronize(&application);

                    let should_exit = false; // FIXME

                    user_interface = ManuallyDrop::new(build_user_interface(
                        &application,
                        cache,
                        &mut renderer,
                        state.logical_size(),
                        &mut debug,
                    ));

                    if should_exit {
                        break;
                    }
                }

                debug.draw_started();
                let new_mouse_interaction = user_interface.draw(
                    &mut renderer,
                    state.theme(),
                    &iced_runtime::core::renderer::Style {
                        text_color: state.text_color(),
                    },
                    state.cursor(),
                );
                debug.draw_finished();

                if new_mouse_interaction != mouse_interaction {
                    // TODO
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
                #[cfg(feature = "trace")]
                let _ = info_span!("Application", "FRAME").entered();

                // Set whenever a baseview event or message gets handled. Or as a stopgap workaround
                // we can also just always redraw.
                if !(redraw_requested || settings.always_redraw) {
                    continue;
                }

                let physical_size = state.physical_size();

                if physical_size.width == 0 || physical_size.height == 0 {
                    continue;
                }

                debug.render_started();
                let current_viewport_version = state.viewport_version();

                if viewport_version != current_viewport_version {
                    let logical_size = state.logical_size();

                    debug.layout_started();
                    user_interface = ManuallyDrop::new(
                        ManuallyDrop::into_inner(user_interface)
                            .relayout(logical_size, &mut renderer),
                    );
                    debug.layout_finished();

                    debug.draw_started();
                    let new_mouse_interaction = user_interface.draw(
                        &mut renderer,
                        state.theme(),
                        &renderer::Style {
                            text_color: state.text_color(),
                        },
                        state.cursor(),
                    );

                    if new_mouse_interaction != mouse_interaction {
                        // window.set_cursor_icon(conversion::mouse_interaction(
                        //     new_mouse_interaction,
                        // ));

                        mouse_interaction = new_mouse_interaction;
                    }
                    debug.draw_finished();

                    compositor.configure_surface(
                        &mut surface,
                        physical_size.width,
                        physical_size.height,
                    );

                    viewport_version = current_viewport_version;
                }

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
                    }
                    Err(error) => match error {
                        // This is an unrecoverable error.
                        compositor::SurfaceError::OutOfMemory => {
                            panic!("{error:?}");
                        }
                        _ => {
                            debug.render_finished();

                            redraw_requested = true;
                        }
                    },
                }
            }
            RuntimeEvent::Baseview ((event, do_send_status)) => {
                state.update(&event, &mut debug);

                let ignore_non_modifier_keys = application
                    .ignore_non_modifier_keys()
                    .unwrap_or(settings.ignore_non_modifier_keys);

                crate::conversion::baseview_to_iced_events(
                    event,
                    &mut events,
                    state.modifiers_mut(),
                    ignore_non_modifier_keys,
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
                    state.cursor(),
                    &mut renderer,
                    &mut clipboard,
                    &mut messages,
                );
                // Will trigger an update when the next frame gets drawn
                needs_update |= matches!(interface_state, user_interface::State::Outdated,);

                if do_send_status {
                    let mut final_status = EventStatus::Ignored;
                    for status in &statuses {
                        if let crate::core::event::Status::Captured = status {
                            final_status = EventStatus::Captured;
                            break;
                        }
                    }

                    *event_status.borrow_mut() = final_status;
                }

                debug.event_processing_finished();

                for (event, status) in events.drain(..).zip(statuses.into_iter()) {
                    runtime.broadcast(event, status);
                }

                did_process_event = true;
            }
            RuntimeEvent::WillClose => {
                if let Some(message) = &window_subs.on_window_will_close {
                    // Send message to user before exiting the loop.

                    messages.push(message());
                    let mut cache = ManuallyDrop::into_inner(user_interface).into_cache();

                    update(
                        &mut application,
                        &mut cache,
                        &state,
                        &mut renderer,
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

    // Manually drop the user interface
    drop(ManuallyDrop::into_inner(user_interface));
}

/// Builds a [`UserInterface`] for the provided [`Application`], logging
/// [`struct@Debug`] information accordingly.
pub fn build_user_interface<'a, A: Application>(
    application: &'a A,
    cache: user_interface::Cache,
    renderer: &mut A::Renderer,
    size: Size,
    debug: &mut Debug,
) -> UserInterface<'a, A::Message, A::Renderer>
where
    <A::Renderer as core::Renderer>::Theme: StyleSheet,
{
    #[cfg(feature = "trace")]
    let view_span = info_span!("Application", "VIEW").entered();

    debug.view_started();
    let view = application.view();

    #[cfg(feature = "trace")]
    let _ = view_span.exit();
    debug.view_finished();

    #[cfg(feature = "trace")]
    let layout_span = info_span!("Application", "LAYOUT").entered();

    debug.layout_started();
    let user_interface = UserInterface::build(view, size, cache, renderer);

    #[cfg(feature = "trace")]
    let _ = layout_span.exit();
    debug.layout_finished();

    user_interface
}

/// Updates an [`Application`] by feeding it the provided messages, spawning any
/// resulting [`Command`], and tracking its [`Subscription`].
pub fn update<A: Application, E: Executor>(
    application: &mut A,
    cache: &mut user_interface::Cache,
    state: &State<A>,
    renderer: &mut A::Renderer,
    runtime: &mut Runtime<E, Proxy<A::Message>, A::Message>,
    clipboard: &mut Clipboard,
    debug: &mut Debug,
    messages: &mut Vec<A::Message>,

    window_subs: &mut WindowSubs<A::Message>,
    window_queue: &mut WindowQueue,
) where
    <A::Renderer as core::Renderer>::Theme: StyleSheet,
{
    for message in messages.drain(..) {
        #[cfg(feature = "trace")]
        let update_span = info_span!("Application", "UPDATE").entered();

        debug.log_message(&message);

        debug.update_started();
        let command = runtime.enter(|| application.update(message));

        #[cfg(feature = "trace")]
        let _ = update_span.exit();
        debug.update_finished();

        run_command(
            application,
            cache,
            state,
            renderer,
            command,
            runtime,
            clipboard,
            debug,
            window_queue
        );
    }

    let subscription = application.subscription(window_subs);
    runtime.track(subscription.into_recipes());
}

/// Runs the actions of a [`Command`].
pub fn run_command<A, E>(
    application: &A,
    cache: &mut user_interface::Cache,
    state: &State<A>,
    renderer: &mut A::Renderer,
    command: Command<A::Message>,
    runtime: &mut Runtime<E, Proxy<A::Message>, A::Message>,
    clipboard: &mut Clipboard,
    debug: &mut Debug,
    window_queue: &mut WindowQueue,
) where
    A: Application,
    E: Executor,
    <A::Renderer as core::Renderer>::Theme: StyleSheet,
{
    use crate::runtime::command;

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
            command::Action::Widget(action) => {
                let mut current_cache = std::mem::take(cache);
                let mut current_operation = Some(action);

                let mut user_interface = build_user_interface(
                    application,
                    current_cache,
                    renderer,
                    state.logical_size(),
                    debug,
                );

                while let Some(mut operation) = current_operation.take() {
                    user_interface.operate(renderer, operation.as_mut());

                    match operation.finish() {
                        operation::Outcome::None => {}
                        operation::Outcome::Some(message) => {
                            runtime.spawn(Box::pin(futures::future::ready(message)));
                        }
                        operation::Outcome::Chain(next) => {
                            current_operation = Some(next);
                        }
                    }
                }

                current_cache = user_interface.into_cache();
                *cache = current_cache;
            }
            command::Action::Window(iced_runtime::window::Action::Close) => {
                if let Err(_) = window_queue.close_window() {
                    debug.log_message(&"could not send close_window command".to_string())
                }
            }
            // Currently not supported
            _ => {}
        }
    }
}


