use baseview::{Event, Window, WindowHandler};
use iced_futures::futures::channel::mpsc;
use iced_graphics::Viewport;
use iced_native::{Debug, Size, Executor, Runtime};
use iced_native::{Cache, UserInterface};
use std::mem::ManuallyDrop;
use std::pin::Pin;
use std::marker::PhantomPinned;

use crate::application;
use crate::{
    Application, Compositor, Parent, Renderer, Settings, WindowScalePolicy, proxy::Proxy,
};

struct PinnedApp<'a, A: Application + 'a + Send> {
    pub application: A,
    pub user_interface: Option<ManuallyDrop<UserInterface<'a, A::Message, Renderer>>>,
    _pin: PhantomPinned,
}

impl<'a, A: Application + 'a + Send> PinnedApp<'a, A> {
    pub fn new(application: A) -> Self {
        Self {
            application,
            user_interface: None,
            _pin: PhantomPinned,
        }
    }
}

/// Handles an iced_baseview application
#[allow(missing_debug_implementations)]
pub struct Runner<'a, A: Application + 'a + Send> {
    pinned_app: Pin<Box<PinnedApp<'a, A>>>,
    state: application::State<A>,
    debug: Debug,
    compositor: Compositor,
    renderer: Renderer,
    surface: <Compositor as iced_graphics::window::Compositor>::Surface,
    swap_chain: <Compositor as iced_graphics::window::Compositor>::SwapChain,
    primitive: (iced_graphics::Primitive, iced_native::mouse::Interaction),
    mouse_interaction: iced_native::mouse::Interaction,
    viewport_version: usize,
    runtime: Runtime<A::Executor, Proxy<A::Message>, A::Message>,
    runtime_rx: mpsc::Receiver<A::Message>,
    events: Vec<iced_native::Event>,
    messages: Vec<A::Message>,
}

impl<'a, A: Application + 'a + Send> Runner<'a, A> {
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
            move |window: &mut baseview::Window<'_>| -> Runner<'a, A> {
                use iced_graphics::window::Compositor as IGCompositor;

                let mut debug = Debug::new();
                debug.startup_started();

                let (runtime_tx, runtime_rx) = mpsc::channel::<A::Message>(128);

                let mut runtime = {
                    let proxy = Proxy::new(runtime_tx);
                    let executor = <A::Executor as Executor>::new().unwrap();
            
                    Runtime::new(executor, proxy)
                };

                let (mut application, init_command) = A::new(settings.flags);

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

                let (mut compositor, mut renderer) =
                    <Compositor as IGCompositor>::new(renderer_settings)
                        .unwrap();

                let surface = compositor.create_surface(window);

                let swap_chain = compositor.create_swap_chain(
                    &surface,
                    physical_size.width,
                    physical_size.height,
                );

                let state = application::State::new(&application, viewport, scale_policy);
                let viewport_version = state.viewport_version();

                let mut pinned_app = Box::pin(PinnedApp::new(application));

                // We know this is safe because modifying a field doesn't move the whole struct.
                let primitive = unsafe {
                    let mut_ref = Pin::as_mut(&mut pinned_app).get_unchecked_mut();

                    let mut user_interface = ManuallyDrop::new(build_user_interface(
                        &mut mut_ref.application,
                        Cache::default(),
                        &mut renderer,
                        state.logical_size(),
                        &mut debug,
                    ));

                    let primitive = user_interface.draw(&mut renderer, state.cursor_position());

                    mut_ref.user_interface = Some(user_interface);

                    primitive
                };

                let mouse_interaction = iced_native::mouse::Interaction::default();

                debug.startup_finished();

                Self {
                    pinned_app,
                    state,
                    debug,
                    compositor,
                    renderer,
                    surface,
                    swap_chain,
                    primitive,
                    mouse_interaction,
                    viewport_version,
                    runtime,
                    runtime_rx,
                    events: Vec::new(),
                    messages: Vec::new(),
                }
            },
        )
    }
}

impl<'a, A: Application + Send> WindowHandler for Runner<'a, A> {
    // TODO: Add message API
    type Message = ();

    fn on_frame(&mut self) {
        use iced_graphics::window::Compositor as IGCompositor;

        self.debug.render_started();
        let current_viewport_version = self.state.viewport_version();

        if self.viewport_version != current_viewport_version {
            let physical_size = self.state.physical_size();
            let logical_size = self.state.logical_size();

            self.debug.layout_started();
            // We know this is safe because modifying a field doesn't move the whole struct.
            unsafe {
                let mut_ref = Pin::as_mut(&mut self.pinned_app).get_unchecked_mut();

                mut_ref.user_interface = Some(ManuallyDrop::new(ManuallyDrop::into_inner(mut_ref.user_interface.take().unwrap())
                    .relayout(logical_size, &mut self.renderer)));
            };
            self.debug.layout_finished();

            self.debug.draw_started();

            self.primitive = self.pinned_app.user_interface.unwrap().draw(&mut self.renderer, self.state.cursor_position());

            /*
            self.primitive = if let Some(user_interface) = self.user_interface {
                user_interface.as_mut().unwrap().draw(&mut self.renderer, self.state.cursor_position())
            } else {
                // This should never happen.
                panic!()
            };
            */
            
            self.debug.draw_finished();

            self.swap_chain = self.compositor.create_swap_chain(
                &self.surface,
                physical_size.width,
                physical_size.height,
            );

            self.viewport_version = current_viewport_version;
        }

        let new_mouse_interaction = self.compositor.draw(
            &mut self.renderer,
            &mut self.swap_chain,
            self.state.viewport(),
            self.state.background_color(),
            &self.primitive,
            &self.debug.overlay(),
        );

        self.debug.render_finished();

        if new_mouse_interaction != self.mouse_interaction {
            // TODO: set baseview mouse cursor

            self.mouse_interaction = new_mouse_interaction;
        }
    }

    fn on_event(&mut self, window: &mut Window<'_>, event: Event) {
        self.state.update(window, &event, &mut self.debug);

        if let Some(iced_event) =
            crate::conversion::baseview_to_iced_event(event)
        {
            self.events.push(iced_event);
        }
    }

    fn on_message(
        &mut self,
        _window: &mut Window<'_>,
        _message: Self::Message,
    ) {
    }
}

impl<'a, A: Application + Send> Drop for Runner<'a, A> {
    fn drop(&mut self) {
        // Manually drop the user interface
        // We know this is safe because modifying a field doesn't move the whole struct.
        unsafe {
            let mut_ref = Pin::as_mut(&mut self.pinned_app).get_unchecked_mut();

            drop(ManuallyDrop::into_inner(mut_ref.user_interface.take().unwrap()));
        };
        
    }
}

/// Builds a [`UserInterface`] for the provided [`Application`], logging
/// [`struct@Debug`] information accordingly.
pub fn build_user_interface<'a, A: Application>(
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