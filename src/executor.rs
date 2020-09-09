use crate::{Application, Settings};

use std::sync::mpsc;

use baseview::{Event, WindowInfo};
use iced_graphics::Viewport;
use iced_native::{program, Color, Command, Debug, Point, Size};
//use iced_wgpu::{wgpu, Backend, Renderer, Viewport};
//use futures::task::SpawnExt;

pub struct Executor<A: Application + 'static> {
    iced_state: program::State<A>,
    cursor_position: Point,
    debug: Debug,
    viewport: Viewport,
    compositor: A::Compositor,
    renderer: A::Renderer,
    surface: <A::Compositor as iced_graphics::window::Compositor>::Surface,
    swap_chain: <A::Compositor as iced_graphics::window::Compositor>::SwapChain,
    background_color: Color,
    redraw_requested: bool,
    window_size: Size<u32>,
    scale_factor: f64,
    resized: bool,
}

impl<A: Application + 'static> Executor<A> {
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
        let _ = baseview::Window::<Executor<A>>::open(
            window_open_options,
            app_message_rx,
        );
    }
}

impl<A: Application + 'static> baseview::AppWindow for Executor<A> {
    type AppMessage = A::AudioToGuiMessage;

    fn build(window: baseview::RawWindow, window_info: &WindowInfo) -> Self {
        use iced_graphics::window::Compositor;

        let mut debug = Debug::new();

        let window_size =
            Size::new(window_info.width as u32, window_info.height as u32);

        let viewport =
            Viewport::with_physical_size(window_size, window_info.scale);

        let compositor_settings = A::compositor_settings();

        let (mut compositor, mut renderer) =
            A::Compositor::new(compositor_settings).unwrap();

        let surface = compositor.create_surface(&window);

        let swap_chain = compositor.create_swap_chain(
            &surface,
            window_size.width,
            window_size.height,
        );

        // Initialize user program
        let (user_program, _initial_command) = A::new();

        let background_color = A::background_color();

        // TODO: do something with `_initial_command`

        // Initialize iced's built-in state
        let iced_state = program::State::new(
            user_program,
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
                self.background_color, // background_color
                self.iced_state.primitive(),
                &self.debug.overlay(),
            );

            /*

            let frame = self.swap_chain.get_current_frame().expect("Next frame");

            let mut encoder = self.iced_renderer.wgpu_context.device.create_command_encoder(
                &wgpu::CommandEncoderDescriptor { label: None },
            );

            let program = self.iced_state.program();

            let mouse_interaction = self.iced_renderer.renderer.backend_mut().draw(
                &mut self.iced_renderer.wgpu_context.device,
                &mut self.iced_renderer.wgpu_context.staging_belt,
                &mut encoder,
                &frame.output.view,
                &self.iced_renderer.viewport,
                self.iced_state.primitive(),
                &self.debug.overlay(),
            );

            // Submit the work
            self.iced_renderer.wgpu_context.staging_belt.finish();
            self.iced_renderer.wgpu_context.queue.submit(Some(encoder.finish()));

            // TODO: set the mouse cursor icon

            // Recall staging buffers
            self.iced_renderer.wgpu_context.local_pool
                .spawner()
                .spawn(self.iced_renderer.wgpu_context.staging_belt.recall())
                .expect("Recall staging buffers");
            self.iced_renderer.wgpu_context.local_pool.run_until_stalled();

            */

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
