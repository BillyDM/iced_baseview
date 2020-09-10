use crate::{Application, Settings};

use std::sync::mpsc;

use baseview::{Event, WindowInfo};
use iced_graphics::Viewport;
use iced_native::{program, Color, Command, Debug, Element, Point, Size};
//use iced_wgpu::{wgpu, Backend, Renderer, Viewport};
//use futures::task::SpawnExt;

#[cfg(feature = "wgpu")]
type Renderer = iced_wgpu::Renderer;

#[cfg(feature = "wgpu")]
type Compositor = iced_wgpu::window::Compositor;

#[cfg(feature = "glow")]
type Renderer = iced_glow::Renderer;

#[cfg(feature = "glow")]
type Compositor = iced_glow::window::Compositor;

struct IcedProgram<A: Application> {
    pub user_app: A,
}

impl<A: Application> iced_native::Program for IcedProgram<A> {
    type Renderer = Renderer;
    type Message = A::Message;

    fn update(&mut self, message: Self::Message) -> Command<Self::Message> {
        self.user_app.update(message)
    }

    fn view(&mut self) -> Element<'_, Self::Message, Self::Renderer> {
        self.user_app.view()
    }
}

pub struct Handler<A: Application + 'static> {
    iced_state: program::State<IcedProgram<A>>,
    cursor_position: Point,
    debug: Debug,
    viewport: Viewport,
    compositor: Compositor,
    renderer: Renderer,
    surface: <Compositor as iced_graphics::window::Compositor>::Surface,
    swap_chain: <Compositor as iced_graphics::window::Compositor>::SwapChain,
    background_color: Color,
    redraw_requested: bool,
    window_size: Size<u32>,
    scale_factor: f64,
    resized: bool,
}

impl<A: Application + 'static> Handler<A> {
    pub fn run(settings: Settings) {
        let window_open_options = baseview::WindowOpenOptions {
            title: settings.window.title.as_str(),
            width: settings.window.size.0 as usize,
            height: settings.window.size.1 as usize,
            parent: baseview::Parent::None,
        };

        // Create channel for sending messages from audio to GUI.
        let (_app_message_tx, app_message_rx) =
            mpsc::channel::<A::AudioToGuiMessage>();

        // Run the baseview window with the executor.
        let _ = baseview::Window::<Handler<A>>::open(
            window_open_options,
            app_message_rx,
        );
    }
}

impl<A: Application + 'static> baseview::AppWindow for Handler<A> {
    type AppMessage = A::AudioToGuiMessage;

    fn build(window: baseview::RawWindow, window_info: &WindowInfo) -> Self {
        use iced_graphics::window::Compositor as IGCompositor;

        let mut debug = Debug::new();

        let window_size =
            Size::new(window_info.width as u32, window_info.height as u32);

        let viewport =
            Viewport::with_physical_size(window_size, window_info.scale);

        let compositor_settings = A::compositor_settings();

        let (mut compositor, mut renderer) =
            <Compositor as IGCompositor>::new(compositor_settings).unwrap();

        let surface = compositor.create_surface(&window);

        let swap_chain = compositor.create_swap_chain(
            &surface,
            window_size.width,
            window_size.height,
        );

        // Initialize user program
        let user_app = A::new();
        let iced_program = IcedProgram { user_app };

        let background_color = A::background_color();

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
            window_size,
            scale_factor: window_info.scale,
            background_color,
            resized: false,
        }
    }

    fn draw(&mut self) {
        println!("draw");

        use iced_graphics::window::Compositor;

        if self.redraw_requested {
            self.debug.render_started();

            if self.resized {
                let physical_size = self.viewport.physical_size();

                self.swap_chain = self.compositor.create_swap_chain(
                    &self.surface,
                    physical_size.width,
                    physical_size.height,
                );

                self.resized = false;
            }

            let _new_mouse_interaction = self.compositor.draw(
                &mut self.renderer,
                &mut self.swap_chain,
                &self.viewport,
                self.background_color,
                self.iced_state.primitive(),
                &self.debug.overlay(),
            );

            self.redraw_requested = false;

            self.debug.render_finished();
        }
    }

    fn on_event(&mut self, event: Event) {
        if let Event::CursorMotion(x, y) = event {
            self.cursor_position.x = x as f32;
            self.cursor_position.y = y as f32;
        }

        if let Event::WindowResized(window_info) = &event {
            if self.window_size.width != window_info.width
                || self.window_size.height != window_info.height
            {
                self.window_size.width = window_info.width;
                self.window_size.height = window_info.height;

                self.viewport = Viewport::with_physical_size(
                    self.window_size,
                    self.scale_factor,
                );

                self.resized = true;
            }
        }

        if let Some(iced_event) =
            crate::conversion::baseview_to_iced_event(event)
        {
            self.iced_state.queue_event(iced_event);

            // update iced
            let _ = self.iced_state.update(
                self.viewport.logical_size(),
                self.cursor_position,
                None, // clipboard
                &mut self.renderer,
                &mut self.debug,
            );

            self.redraw_requested = true;
        }
    }

    fn on_app_message(&mut self, message: Self::AppMessage) {}
}
