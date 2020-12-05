use crate::application::Instance;
use crate::{
    Application, Compositor, Parent, Renderer, Settings, WindowScalePolicy,
};

use baseview::{Event, MouseEvent, Window, WindowEvent, WindowHandler};
use iced_graphics::Viewport;
use iced_native::{program, Color, Debug, Point, Size};

/// Handles an iced_baseview application
#[allow(missing_debug_implementations)]
pub struct Runner<A: Application + 'static + Send> {
    iced_state: program::State<Instance<A>>,
    cursor_position: Point,
    debug: Debug,
    viewport: Viewport,
    compositor: Compositor,
    renderer: Renderer,
    surface: <Compositor as iced_graphics::window::Compositor>::Surface,
    swap_chain: <Compositor as iced_graphics::window::Compositor>::SwapChain,
    background_color: Color,
    redraw_requested: bool,
    physical_size: Size<u32>,
    scale_policy: WindowScalePolicy,
}

impl<A: Application + 'static + Send> Runner<A> {
    /// Open a new window
    pub fn open(
        settings: Settings<A::Flags>,
        parent: Parent,
    ) -> (baseview::WindowHandle<Self>, Option<baseview::AppRunner>){
        // TODO: use user_command
        let (user_app, _user_command) = A::new(settings.flags);

        // TODO: WindowScalePolicy should derive copy or clone.
        let scale_policy = match settings.window.scale {
            WindowScalePolicy::ScaleFactor(scale) => {
                WindowScalePolicy::ScaleFactor(scale)
            }
            WindowScalePolicy::SystemScaleFactor => {
                WindowScalePolicy::SystemScaleFactor
            }
        };

        let logical_width = settings.window.logical_size.0 as f64;
        let logical_height = settings.window.logical_size.1 as f64;

        let window_settings = baseview::WindowOpenOptions {
            title: user_app.title(),
            size: baseview::Size::new(logical_width, logical_height),
            scale: settings.window.scale,
            parent,
        };

        Window::open(
            window_settings,
            move |window: &mut baseview::Window<'_>| -> Runner<A> {
                use iced_graphics::window::Compositor as IGCompositor;

                let mut debug = Debug::new();

                let background_color = user_app.background_color();

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

                let iced_program = Instance(user_app);

                // Initialize iced's built-in state
                let iced_state = program::State::new(
                    iced_program,
                    viewport.logical_size(),
                    Point::new(-1.0, -1.0),
                    &mut renderer,
                    &mut debug,
                );

                Self {
                    iced_state,
                    cursor_position: Point::new(-1.0, -1.0),
                    debug,
                    viewport,
                    compositor,
                    renderer,
                    surface,
                    swap_chain,
                    redraw_requested: true,
                    physical_size,
                    background_color,
                    scale_policy,
                }
            },
        )
    }
}

impl<A: Application + 'static + Send> WindowHandler for Runner<A> {
    // TODO: Add message API
    type Message = ();

    fn on_frame(&mut self) {
        use iced_graphics::window::Compositor as IGCompositor;

        if self.redraw_requested {
            // Update iced state
            let _ = self.iced_state.update(
                self.viewport.logical_size(),
                self.cursor_position,
                None, // clipboard
                &mut self.renderer,
                &mut self.debug,
            );

            self.debug.render_started();

            let _new_mouse_interaction = self.compositor.draw(
                &mut self.renderer,
                &mut self.swap_chain,
                &self.viewport,
                self.background_color,
                self.iced_state.primitive(),
                &self.debug.overlay(),
            );

            self.debug.render_finished();

            self.redraw_requested = false;
        }
    }

    fn on_event(&mut self, _window: &mut Window<'_>, event: Event) {
        use iced_graphics::window::Compositor as IGCompositor;

        if let Event::Mouse(MouseEvent::CursorMoved { position }) = event {
            self.cursor_position =
                Point::new(position.x as f32, position.y as f32);
        };

        if let Event::Window(WindowEvent::Resized(window_info)) = event {
            self.physical_size.width = window_info.physical_size().width;
            self.physical_size.height = window_info.physical_size().height;

            let scale = match self.scale_policy {
                WindowScalePolicy::ScaleFactor(scale) => scale,
                WindowScalePolicy::SystemScaleFactor => window_info.scale(),
            };

            self.viewport =
                Viewport::with_physical_size(self.physical_size, scale);

            self.swap_chain = self.compositor.create_swap_chain(
                &self.surface,
                self.physical_size.width,
                self.physical_size.height,
            );
        }

        if let Some(iced_event) =
            crate::conversion::baseview_to_iced_event(event)
        {
            self.iced_state.queue_event(iced_event);

            self.redraw_requested = true;
        }
    }

    fn on_message(
        &mut self,
        _window: &mut Window<'_>,
        _message: Self::Message,
    ) {
    }
}
