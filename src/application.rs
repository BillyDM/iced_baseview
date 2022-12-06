//! Create interactive, native cross-platform applications.
mod state;

use baseview::{EventStatus, Window, WindowScalePolicy};
use iced_native::event::Status;
use iced_native::widget::operation;
use iced_native::{mouse, Command, Debug, Executor, Runtime, Size, Subscription};
pub use state::State;

use crate::clipboard::{self, Clipboard};
use crate::settings::IcedBaseviewSettings;
use crate::window::{IcedWindow, RuntimeEvent, WindowQueue, WindowSubs};
use crate::{error::Error, proxy::Proxy, Settings};

use iced_futures::futures;
use iced_futures::futures::channel::mpsc;
use iced_graphics::{compositor, Viewport};
use iced_native::user_interface::{self, UserInterface};

pub use iced_native::application::{Appearance, StyleSheet};

use std::cell::RefCell;
use std::mem::ManuallyDrop;
use std::rc::Rc;

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
pub trait Application: Sized + Send
where
    <Self::Renderer as iced_native::Renderer>::Theme: StyleSheet,
{
    /// The data needed to initialize your [`Application`].
    type Flags: Send;

    /// The graphics backend to use to draw the [`Program`].
    type Renderer: iced_native::Renderer;

    /// The type of __messages__ your [`Program`] will produce.
    type Message: std::fmt::Debug + Send;

    /// Initializes the [`Application`] with the flags provided to
    /// [`run`] as part of the [`Settings`].
    ///
    /// Here is where you should return the initial state of your app.
    ///
    /// Additionally, you can return a [`Command`] if you need to perform some
    /// async action in the background on startup. This is useful if you want to
    /// load state from a file, perform an initial HTTP request, etc.
    fn new(flags: Self::Flags) -> (Self, iced_native::Command<Self::Message>);

    /// Returns the current title of the [`Application`].
    ///
    /// This title can be dynamic! The runtime will automatically update the
    /// title of your application when necessary.
    fn title(&self) -> String;

    /// Returns the current `Theme` of the [`Application`].
    fn theme(&self) -> <Self::Renderer as iced_native::Renderer>::Theme;

    /// Returns the `Style` variation of the `Theme`.
    fn style(&self) -> <<Self::Renderer as iced_native::Renderer>::Theme as StyleSheet>::Style {
        Default::default()
    }

    /// Handles a __message__ and updates the state of the [`Program`].
    ///
    /// This is where you define your __update logic__. All the __messages__,
    /// produced by either user interactions or commands, will be handled by
    /// this method.
    ///
    /// Any [`Command`] returned will be executed immediately in the
    /// background by shells.
    fn update(
        &mut self,
        window: &mut WindowQueue,
        message: Self::Message,
    ) -> iced_native::Command<Self::Message>;

    /// Returns the widgets to display in the [`Program`].
    ///
    /// These widgets can produce __messages__ based on user interaction.
    fn view(&self) -> iced_native::Element<'_, Self::Message, Self::Renderer>;

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

    /// Returns whether the [`Application`] should be terminated.
    ///
    /// By default, it returns `false`.
    fn should_exit(&self) -> bool {
        false
    }

    /// Returns the [`WindowScalePolicy`] that the [`Application`] should use.
    ///
    /// By default, it returns `WindowScalePolicy::SystemScaleFactor`.
    ///
    /// [`WindowScalePolicy`]: ../settings/enum.WindowScalePolicy.html
    /// [`Application`]: trait.Application.html
    fn scale_policy(&self) -> WindowScalePolicy {
        WindowScalePolicy::SystemScaleFactor
    }

    fn renderer_settings() -> crate::renderer::Settings;
}

/// Runs an [`Application`] with an executor, compositor, and the provided
/// settings.
pub(crate) fn run<A, E, C>(
    window: &mut Window<'_>,
    settings: Settings<A::Flags>,
    sender: mpsc::UnboundedSender<RuntimeEvent<A::Message>>,
    receiver: mpsc::UnboundedReceiver<RuntimeEvent<A::Message>>,
) -> Result<IcedWindow<A>, Error>
where
    A: Application + Send + 'static,
    E: Executor + 'static,
    C: crate::IGCompositor<Renderer = A::Renderer, Settings = crate::renderer::Settings> + 'static,
    <A::Renderer as iced_native::Renderer>::Theme: StyleSheet,
{
    use futures::task;

    let mut debug = Debug::new();
    debug.startup_started();

    let viewport = {
        // Assume scale for now until there is an event with a new one.
        let scale = match settings.window.scale {
            WindowScalePolicy::ScaleFactor(scale) => scale,
            WindowScalePolicy::SystemScaleFactor => 1.0,
        };

        let physical_size = Size::new(
            (settings.window.size.width * scale) as u32,
            (settings.window.size.height * scale) as u32,
        );

        Viewport::with_physical_size(physical_size, scale)
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

    let state = State::new(&application, viewport.clone(), settings.window.scale);
    let clipboard = Clipboard::connect(&window);

    let renderer_settings = A::renderer_settings();

    cfg_if::cfg_if! {
        if #[cfg(feature = "wgpu")] {
            let window = crate::wrapper::WindowHandleWrapper(window);
            let (mut compositor, renderer) =
                C::new(renderer_settings, Some(&window)).unwrap();
            let surface = compositor.create_surface(&window);
        } else {
            let (compositor, renderer) = {
                let context = window
                    .gl_context()
                    .expect("Window was created without OpenGL support");
                unsafe { context.make_current() };

                let (compositor, renderer) = unsafe {
                    C::new(renderer_settings, |s| {
                        context.get_proc_address(s)
                    })
                    .unwrap()
                };

                unsafe { context.make_not_current() };

                (compositor, renderer)
            };
        }
    }

    let event_status = Rc::new(RefCell::new(EventStatus::Ignored));
    let (window_queue, window_queue_rx) = WindowQueue::new();

    // let (sender, receiver) = mpsc::unbounded();

    let instance = Box::pin(run_instance::<A, E, C>(
        application,
        compositor,
        renderer,
        runtime,
        // proxy,
        debug,
        receiver,
        init_command,
        // window,
        // settings.exit_on_close_request,
        state,
        #[cfg(feature = "wgpu")]
        surface,
        clipboard,
        settings.iced_baseview,
        event_status.clone(),
        window_queue,
    ));

    let runtime_context = task::Context::from_waker(task::noop_waker_ref());

    Ok(IcedWindow {
        sender,
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
    // mut proxy: winit::event_loop::EventLoopProxy<A::Message>,
    mut debug: Debug,
    mut receiver: mpsc::UnboundedReceiver<RuntimeEvent<A::Message>>,
    init_command: Command<A::Message>,
    // window: Window<'_>,
    // exit_on_close_request: bool,
    mut state: State<A>,
    #[cfg(feature = "wgpu")] mut surface: C::Surface,
    mut clipboard: Clipboard,
    settings: IcedBaseviewSettings,
    event_status: Rc<RefCell<EventStatus>>,
    mut window_queue: WindowQueue,
) where
    A: Application + 'static,
    E: Executor + 'static,
    C: crate::IGCompositor<Renderer = A::Renderer> + 'static,
    <A::Renderer as iced_native::Renderer>::Theme: StyleSheet,
{
    use iced_futures::futures::stream::StreamExt;

    let mut cache = user_interface::Cache::default();
    let mut viewport_version = state.viewport_version();
    let mut window_subs = WindowSubs::default();

    #[cfg(feature = "wgpu")]
    {
        let physical_size = state.physical_size();

        compositor.configure_surface(&mut surface, physical_size.width, physical_size.height);
    }

    run_command(
        &application,
        &mut cache,
        &state,
        &mut renderer,
        init_command,
        &mut runtime,
        &mut clipboard,
        // &mut proxy,
        &mut debug,
        // &window,
        || compositor.fetch_information(),
    );
    runtime.track(application.subscription(&mut window_subs));

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

    while let Some(event) = receiver.next().await {
        match event {
            RuntimeEvent::Baseview((event, do_send_status)) => {
                state.update(&event, &mut debug);

                crate::conversion::baseview_to_iced_events(
                    event,
                    &mut events,
                    state.modifiers_mut(),
                    settings.ignore_non_modifier_keys,
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
                needs_update |= matches!(interface_state, user_interface::State::Outdated,);

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
                        state.cursor_position(),
                        &mut renderer,
                        &mut clipboard,
                        &mut messages,
                    );

                    needs_update |= matches!(interface_state, user_interface::State::Outdated,);

                    debug.event_processing_finished();

                    for event in events.drain(..).zip(statuses.into_iter()) {
                        runtime.broadcast(event);
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
                        // &mut proxy,
                        &mut debug,
                        &mut messages,
                        // &window,
                        || compositor.fetch_information(),
                        &mut window_subs,
                        &mut window_queue,
                    );

                    // Update window
                    state.synchronize(&application);

                    let should_exit = application.should_exit();

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
                    &iced_native::renderer::Style {
                        text_color: state.text_color(),
                    },
                    state.cursor_position(),
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
                        &iced_native::renderer::Style {
                            text_color: state.text_color(),
                        },
                        state.cursor_position(),
                    );

                    if new_mouse_interaction != mouse_interaction {
                        // TODO
                        // window.set_cursor_icon(conversion::mouse_interaction(
                        //     new_mouse_interaction,
                        // ));

                        mouse_interaction = new_mouse_interaction;
                    }
                    debug.draw_finished();

                    cfg_if::cfg_if! {
                        if #[cfg(feature = "wgpu")] {
                            compositor.configure_surface(
                                &mut surface,
                                physical_size.width,
                                physical_size.height,
                            );
                        } else {
                            compositor.resize_viewport(physical_size);
                        }
                    }

                    viewport_version = current_viewport_version;
                }

                cfg_if::cfg_if! {
                    if #[cfg(feature = "wgpu")] {
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
                                iced_graphics::compositor::SurfaceError::OutOfMemory => {
                                    panic!("{:?}", error);
                                }
                                _ => {
                                    debug.render_finished();

                                    // Try rendering again next frame.
                                    redraw_requested = true;
                                }
                            },
                        }
                    } else {
                        // The buffer swap happens in `IcedWindow::on_frame()` for the glow backend
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
                }
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
                        // &mut proxy,
                        &mut debug,
                        &mut messages,
                        // &window,
                        || compositor.fetch_information(),
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
    <A::Renderer as iced_native::Renderer>::Theme: StyleSheet,
{
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
    cache: &mut user_interface::Cache,
    state: &State<A>,
    renderer: &mut A::Renderer,
    runtime: &mut Runtime<E, Proxy<A::Message>, A::Message>,
    clipboard: &mut Clipboard,
    debug: &mut Debug,
    messages: &mut Vec<A::Message>,
    graphics_info: impl FnOnce() -> compositor::Information + Copy,

    window_subs: &mut WindowSubs<A::Message>,
    window_queue: &mut WindowQueue,
) where
    <A::Renderer as iced_native::Renderer>::Theme: StyleSheet,
{
    for message in messages.drain(..) {
        debug.log_message(&message);

        debug.update_started();
        let command = runtime.enter(|| application.update(window_queue, message));
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
            graphics_info,
        );
    }

    let subscription = application.subscription(window_subs);
    runtime.track(subscription);
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
    _graphics_info: impl FnOnce() -> compositor::Information + Copy,
) where
    A: Application,
    E: Executor,
    <A::Renderer as iced_native::Renderer>::Theme: StyleSheet,
{
    use iced_native::command;

    for action in command.actions() {
        match action {
            command::Action::Future(future) => {
                runtime.spawn(future);
            }
            command::Action::Clipboard(action) => match action {
                clipboard::Action::Read(tag) => {
                    let message = tag(clipboard.read());

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
                let mut current_operation = Some(action.into_operation());

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
            // Currently not supported
            command::Action::Window(_) | command::Action::System(_) => {}
        }
    }
}
