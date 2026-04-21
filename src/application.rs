//! Create interactive, native cross-platform applications.
#[cfg(feature = "trace")]
mod profiler;
mod state;

use baseview::EventStatus;

use iced_debug::Span;
use iced_runtime::Action;
use iced_runtime::Task;
use iced_widget::core::Color;
use iced_widget::core::Element;
use iced_widget::Theme;
use raw_window_handle::HasRawDisplayHandle;
pub use state::State;

use crate::core::renderer;
use crate::core::widget::operation;
use crate::core::Size;
use crate::futures::futures;
use crate::futures::{Executor, Runtime, Subscription};
use crate::graphics::compositor::{self, Compositor};
use crate::runtime::clipboard;
use crate::runtime::user_interface::{self, UserInterface};
use crate::window::{IcedWindow, RuntimeEvent, WindowQueue, WindowSubs};
use crate::{Clipboard, Error, Proxy, Renderer, Settings};

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
pub trait Application
where
    Self: Sized,
    Self::Theme: DefaultStyle,
{
    /// The type of __messages__ your [`Application`] will produce.
    type Message: std::fmt::Debug + Send + 'static;

    /// The theme used to draw the [`Application`].
    type Theme: Default + DefaultStyle;

    /// The [`Executor`] that will run commands and subscriptions.
    ///
    /// The [default executor] can be a good starting point!
    ///
    /// [`Executor`]: Self::Executor
    /// [default executor]: crate::futures::backend::default::Executor
    type Executor: Executor;

    /// The data needed to initialize your [`Application`].
    type Flags;

    /// Initializes the [`Application`] with the flags provided to
    /// [`run`] as part of the [`Settings`].
    ///
    /// Here is where you should return the initial state of your app.
    ///
    /// Additionally, you can return a [`Task`] if you need to perform some
    /// async action in the background on startup. This is useful if you want to
    /// load state from a file, perform an initial HTTP request, etc.
    fn new(flags: Self::Flags) -> (Self, Task<Self::Message>);

    /// Returns the current title of the [`Application`].
    ///
    /// This title can be dynamic! The runtime will automatically update the
    /// title of your application when necessary.
    fn title(&self) -> String {
        "iced_baseview".into()
    }

    /// Handles a __message__ and updates the state of the [`Application`].
    ///
    /// This is where you define your __update logic__. All the __messages__,
    /// produced by either user interactions or commands, will be handled by
    /// this method.
    ///
    /// Any [`Task`] returned will be executed immediately in the background by the
    /// runtime.
    fn update(&mut self, message: Self::Message) -> Task<Self::Message>;

    /// Returns the widgets to display in the [`Application`] for the main window.
    ///
    /// These widgets can produce __messages__ based on user interaction.
    fn view(&self) -> Element<'_, Self::Message, Self::Theme, Renderer>;

    /// Returns the current `Theme` of the [`Application`].
    fn theme(&self) -> Self::Theme;

    /// Returns the `Style` variation of the `Theme`.
    fn style(&self, theme: &Self::Theme) -> Appearance {
        theme.default_style()
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

    /// Returns the [`WindowScalePolicy`] that the [`Application`] should use.
    ///
    /// By default, it returns `WindowScalePolicy::SystemScaleFactor`.
    ///
    /// [`WindowScalePolicy`]: ../settings/enum.WindowScalePolicy.html
    /// [`Application`]: trait.Application.html
    fn scale_policy(&self) -> baseview::WindowScalePolicy {
        baseview::WindowScalePolicy::SystemScaleFactor
    }

    /// Ignore non-modifier keyboard keys. Overrides the field in
    /// `IcedBaseviewSettings` if set
    fn ignore_non_modifier_keys(&self) -> Option<bool> {
        None
    }

    //fn renderer_settings() -> crate::renderer::Settings;
}

/// The appearance of a application.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Appearance {
    /// The background [`Color`] of the application.
    pub background_color: Color,

    /// The default text [`Color`] of the application.
    pub text_color: Color,
}

/// The default style of an [`Application`].
pub trait DefaultStyle {
    /// Returns the default style of an [`Application`].
    fn default_style(&self) -> Appearance;
}

impl DefaultStyle for Theme {
    fn default_style(&self) -> Appearance {
        default(self)
    }
}

/// The default [`Appearance`] of an [`Application`] with the built-in [`Theme`].
pub fn default(theme: &Theme) -> Appearance {
    let palette = theme.extended_palette();

    Appearance {
        background_color: palette.background.base.color,
        text_color: palette.background.base.text,
    }
}

/// Runs an [`Application`] with an executor, compositor, and the provided
/// settings.
pub(crate) fn run<A, C>(
    window: &mut baseview::Window<'_>,
    flags: A::Flags,
    settings: Settings,
    event_sender: mpsc::UnboundedSender<RuntimeEvent<A::Message>>,
    event_receiver: mpsc::UnboundedReceiver<RuntimeEvent<A::Message>>,
) -> Result<IcedWindow<A>, Error>
where
    A: Application + 'static + Send,
    C: Compositor<Renderer = Renderer> + 'static,
    A::Theme: DefaultStyle,
{
    use futures::task;

    #[cfg(feature = "trace")]
    let _guard = Profiler::init();

    let boot_trace = iced_debug::boot();

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

    let (runtime_tx, runtime_rx) = mpsc::unbounded::<Action<A::Message>>();

    let mut runtime = {
        let proxy = Proxy::new(runtime_tx);
        let executor = A::Executor::new().map_err(Error::ExecutorCreationFailed)?;

        Runtime::new(executor, proxy)
    };

    let (application, init_task) = runtime.enter(|| A::new(flags));

    if let Some(stream) = crate::runtime::task::into_stream(init_task) {
        runtime.run(stream);
    }

    let mut window_subs = WindowSubs::default();

    runtime.track(crate::futures::subscription::into_recipes(runtime.enter(
        || {
            application
                .subscription(&mut window_subs)
                .map(Action::Output)
        },
    )));

    let window06 = crate::conversion::convert_window(window);

    let graphics_settings = settings.graphics_settings;
    let mut compositor = runtime.block_on(C::new(graphics_settings, window06.clone()))?;
    let surface = compositor.create_surface(
        window06,
        viewport.physical_width(),
        viewport.physical_height(),
    );
    let renderer = compositor.create_renderer();

    for font in settings.fonts {
        compositor.load_font(font);
    }

    let (window_queue, window_queue_rx) = WindowQueue::new();
    let event_status = Rc::new(RefCell::new(baseview::EventStatus::Ignored));

    let state = State::new(&application, viewport);

    let display_handle = crate::conversion::convert_raw_display_handle(window.raw_display_handle());
    let clipboard = Clipboard::new(display_handle);

    let instance = Box::pin({
        let run_instance = run_instance::<A, C>(
            application,
            compositor,
            renderer,
            runtime,
            event_receiver,
            clipboard,
            window_subs,
            settings.iced_baseview,
            surface,
            event_status.clone(),
            state,
            window_queue,
            boot_trace,
        );

        #[cfg(feature = "trace")]
        let run_instance = run_instance.instrument(info_span!("Application", "LOOP"));

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

#[allow(clippy::too_many_arguments)]
async fn run_instance<A, C>(
    mut application: A,
    mut compositor: C,
    mut renderer: Renderer,
    mut runtime: Runtime<A::Executor, Proxy<A::Message>, iced_runtime::Action<A::Message>>,
    mut event_receiver: mpsc::UnboundedReceiver<RuntimeEvent<A::Message>>,
    mut clipboard: Clipboard,
    mut window_subs: WindowSubs<<A as Application>::Message>,

    settings: crate::settings::IcedBaseviewSettings,
    mut surface: C::Surface,
    event_status: Rc<RefCell<baseview::EventStatus>>,
    mut state: State<A>,
    mut window_queue: WindowQueue,
    boot_trace: Span,
) where
    // What an absolute monstrosity of generics.
    C: Compositor<Renderer = Renderer> + 'static,
    A: Application + 'static,
    A::Theme: DefaultStyle,
{
    use futures::stream::StreamExt;

    let mut viewport_version = state.viewport_version();

    let cache = user_interface::Cache::default();
    let mut events = Vec::new();
    let mut messages = Vec::new();

    let window_id = crate::window::Id::unique();

    let mut user_interface = ManuallyDrop::new(build_user_interface(
        &application,
        cache,
        &mut renderer,
        state.logical_size(),
        window_id,
    ));

    // Triggered whenever a baseview event gets sent
    let mut redraw_requested = true;
    // May be triggered when processing baseview events, will cause the UI to be updated in the next
    // frame
    let mut needs_update = true;
    let mut did_process_event = false;

    boot_trace.finish();

    let mut render_span = None;

    loop {
        // Empty the queue if possible
        let event = if let Ok(event) = event_receiver.try_next() {
            event
        } else {
            event_receiver.next().await
        };

        let Some(event) = event else {
            break;
        };

        match event {
            RuntimeEvent::MainEventsCleared => {
                if let Some(message) = &window_subs.on_frame {
                    if let Some(message) = message() {
                        messages.push(message);
                    }
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
                    let interact_time = iced_debug::interact(window_id);
                    let (interface_state, statuses) = user_interface.update(
                        &events,
                        state.cursor(),
                        &mut renderer,
                        &mut clipboard,
                        &mut messages,
                    );

                    needs_update |= matches!(interface_state, user_interface::State::Outdated,);

                    for (event, status) in events.drain(..).zip(statuses.into_iter()) {
                        runtime.broadcast(crate::futures::subscription::Event::Interaction {
                            window: window_id,
                            event,
                            status,
                        });
                    }
                    interact_time.finish();
                }

                // The user interface update may have pushed a new message onto the stack
                needs_update |= !messages.is_empty() || settings.always_redraw;

                if needs_update {
                    needs_update = false;

                    let cache = ManuallyDrop::into_inner(user_interface).into_cache();

                    // Update application
                    update(
                        &mut application,
                        &mut runtime,
                        &mut messages,
                        &mut window_subs,
                        //&mut window_queue,
                    );

                    // Update window
                    state.synchronize(&application);

                    let should_exit = false; // FIXME

                    user_interface = ManuallyDrop::new(build_user_interface(
                        &application,
                        cache,
                        &mut renderer,
                        state.logical_size(),
                        window_id,
                    ));

                    if should_exit {
                        break;
                    }
                }

                render_span = Some(iced_debug::draw(window_id));
                user_interface.draw(
                    &mut renderer,
                    state.theme(),
                    &iced_runtime::core::renderer::Style {
                        text_color: state.text_color(),
                    },
                    state.cursor(),
                );

                redraw_requested = true;
            }
            RuntimeEvent::UserEvent(message) => {
                run_action::<A, C>(
                    message,
                    &mut compositor,
                    &renderer,
                    &mut messages,
                    &mut clipboard,
                    &mut user_interface,
                    &mut window_queue,
                );
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

                let current_viewport_version = state.viewport_version();

                if viewport_version != current_viewport_version {
                    let logical_size = state.logical_size();

                    let layout_span = iced_debug::layout(window_id);
                    user_interface = ManuallyDrop::new(
                        ManuallyDrop::into_inner(user_interface)
                            .relayout(logical_size, &mut renderer),
                    );
                    layout_span.finish();

                    let draw_span = iced_debug::draw(window_id);
                    user_interface.draw(
                        &mut renderer,
                        state.theme(),
                        &renderer::Style {
                            text_color: state.text_color(),
                        },
                        state.cursor(),
                    );
                    draw_span.finish();

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
                    || {},
                ) {
                    Ok(()) => {
                        // TODO: Handle animations!
                        // Maybe we can use `ControlFlow::WaitUntil` for this.
                        if let Some(span) = render_span {
                            span.finish();
                            render_span = None;
                        }
                    }
                    Err(error) => match error {
                        // This is an unrecoverable error.
                        compositor::SurfaceError::OutOfMemory => {
                            panic!("{error:?}");
                        }
                        _ => {
                            redraw_requested = true;
                        }
                    },
                }
            }
            RuntimeEvent::Baseview((event, do_send_status)) => {
                state.update(&event);

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

                did_process_event = true;
            }
            RuntimeEvent::WillClose => {
                if let Some(message) = &window_subs.on_window_will_close {
                    // Send message to user before exiting the loop.

                    if let Some(message) = message() {
                        messages.push(message);
                    }
                    let cache = ManuallyDrop::into_inner(user_interface).into_cache();

                    update(
                        &mut application,
                        &mut runtime,
                        &mut messages,
                        &mut window_subs,
                    );

                    // Update window
                    state.synchronize(&application);

                    user_interface = ManuallyDrop::new(build_user_interface(
                        &application,
                        cache,
                        &mut renderer,
                        state.logical_size(),
                        window_id,
                    ));
                }

                break;
            }
        }
    }

    // Manually drop the user interface
    let _ = ManuallyDrop::into_inner(user_interface);
}

/// Builds a [`UserInterface`] for the provided [`Application`], logging
/// [`struct@Debug`] information accordingly.
pub fn build_user_interface<'a, A: Application>(
    application: &'a A,
    cache: user_interface::Cache,
    renderer: &mut Renderer,
    size: Size,
    window_id: crate::window::Id,
) -> UserInterface<'a, A::Message, A::Theme, Renderer>
where
    A::Theme: DefaultStyle,
{
    #[cfg(feature = "trace")]
    let view_span = info_span!("Application", "VIEW").entered();

    let view_span = iced_debug::view(window_id);
    let view = application.view();
    view_span.finish();

    #[cfg(feature = "trace")]
    let _ = view_span.exit();

    #[cfg(feature = "trace")]
    let layout_span = info_span!("Application", "LAYOUT").entered();

    let layout_span = iced_debug::layout(window_id);
    let user_interface = UserInterface::build(view, size, cache, renderer);
    layout_span.finish();

    #[cfg(feature = "trace")]
    let _ = layout_span.exit();

    user_interface
}

/// Updates an [`Application`] by feeding it the provided messages, spawning any
/// resulting [`Command`], and tracking its [`Subscription`].
pub fn update<A: Application, E: Executor>(
    application: &mut A,
    runtime: &mut Runtime<E, Proxy<A::Message>, iced_runtime::Action<A::Message>>,
    messages: &mut Vec<A::Message>,
    window_subs: &mut WindowSubs<A::Message>,
    //window_queue: &mut WindowQueue,
) where
    A::Theme: DefaultStyle,
{
    for message in messages.drain(..) {
        #[cfg(feature = "trace")]
        let update_span = info_span!("Application", "UPDATE").entered();

        let task = runtime.enter(|| application.update(message));

        #[cfg(feature = "trace")]
        let _ = update_span.exit();

        if let Some(stream) = crate::runtime::task::into_stream(task) {
            runtime.run(stream);
        }
    }

    let subscription = runtime.enter(|| application.subscription(window_subs));
    runtime.track(crate::futures::subscription::into_recipes(
        subscription.map(Action::Output),
    ));
}

/// Runs the actions of a [`Command`].
pub fn run_action<A, C>(
    action: Action<A::Message>,
    compositor: &mut C,
    renderer: &Renderer,
    messages: &mut Vec<A::Message>,
    clipboard: &mut Clipboard,
    interface: &mut UserInterface<'_, A::Message, A::Theme, Renderer>,
    window_queue: &mut WindowQueue,
) where
    C: Compositor<Renderer = Renderer> + 'static,
    A: Application + 'static,
    A::Theme: DefaultStyle,
{
    use iced_runtime::window::Action as IWindowAction;

    match action {
        Action::Output(message) => {
            messages.push(message);
        }
        Action::Clipboard(action) => match action {
            clipboard::Action::Read { target, channel } => {
                let _ = channel.send(clipboard.read(target));
            }
            clipboard::Action::Write { target, contents } => {
                clipboard.write(target, contents);
            }
        },
        Action::Window(action) => match action {
            IWindowAction::Close(_) => {
                let _ = window_queue.close_window();
            }
            IWindowAction::Resize(_, size) => {
                let _ = window_queue.resize_window(size);
            }
            IWindowAction::GainFocus(_) => {
                let _ = window_queue.focus();
            }
            _ => {}
        },
        Action::System(action) => match action {
            crate::runtime::system::Action::QueryInformation(_channel) => {
                #[cfg(feature = "system")]
                {
                    let graphics_info = compositor.fetch_information();

                    let _ = std::thread::spawn(move || {
                        let information = crate::system::information(graphics_info);

                        let _ = _channel.send(information);
                    });
                }
            }
        },
        Action::Widget(operation) => {
            let mut current_operation = Some(operation);

            while let Some(mut operation) = current_operation.take() {
                interface.operate(renderer, operation.as_mut());

                match operation.finish() {
                    operation::Outcome::None => {}
                    operation::Outcome::Some(()) => {}
                    operation::Outcome::Chain(next) => {
                        current_operation = Some(next);
                    }
                }
            }
        }
        Action::LoadFont { bytes, channel } => {
            // TODO: Error handling (?)
            compositor.load_font(bytes.clone());

            let _ = channel.send(Ok(()));
        }
        Action::Exit => {
            // ignore errors when closing
            let _ = window_queue.close_window();
        }
        Action::Reload => todo!(),
    }
}
