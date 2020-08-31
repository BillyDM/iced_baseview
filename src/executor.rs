// This is where baseview will interact with the plugin GUI.

use crate::{Application, Settings};

use crate::{mouse, Clipboard, Color, Command, Debug, Point, Runtime, Size, Subscription};

use iced_graphics::window;
use iced_graphics::Viewport;
use iced_native::program::{self, Program};

#[allow(missing_debug_implementations)]
pub struct Executor<Gui, Comp>
where
    Gui: Application + 'static,
    Comp: window::Compositor<Renderer = Gui::Renderer>,
{
    state: program::State<Gui>,
    compositor: Comp,
    renderer: Comp::Renderer,
    viewport: Viewport,
    debug: Debug,
    background_color: Color,
    scale_factor: f64,
    window_size: Size<u32>,
    cursor_position: Point,
    window_focused: bool,
    mouse_interaction: mouse::Interaction,
    resized: bool,
    redraw_requested: bool,
}

impl<Gui, Comp> Executor<Gui, Comp>
where
    Gui: Application + 'static,
    Comp: window::Compositor<Renderer = Gui::Renderer>,
{
    /// Creates a new [`Executor`]
    ///
    /// [`Executor`]: struct.Executor.html
    pub fn new(
        settings: &Settings<Gui::Flags>,
        // window handle as argument
    ) -> Self {
        let mut debug = Debug::new();
        debug.startup_started();

        let window_size = Size::new(settings.window.size.0, settings.window.size.1);

        let bounds = Size::new(settings.window.size.0 as f32, settings.window.size.1 as f32);

        let (mut gui, init_command) = Gui::new(settings);
        // todo: store `init_command` to future queue

        let background_color = gui.background_color();
        let scale_factor = gui.scale_factor();
        let mouse_interaction = mouse::Interaction::default();

        let viewport = Viewport::with_physical_size(window_size, scale_factor);
        let physical_size = viewport.physical_size();

        // Add this as a list of subscriptions the runtime
        // should keep track of.
        let subscription = gui.subscription();

        let compositor_settings = Comp::Settings::default();

        // The wgpu compositor and renderer
        let (mut compositor, mut renderer) = Comp::new(compositor_settings);

        /*
        // window is a window handle
        let surface = compositor.create_surface(&window);

        let mut swap_chain = compositor.create_swap_chain(
            &surface,
            physical_size.width,
            physical_size.height,
        );
        */

        // retrieve initial window state from baseview
        let cursor_position = Point::new(0.0, 0.0);
        let window_focused = true;

        // The state of the iced program.
        let state = program::State::new(gui, bounds, cursor_position, &mut renderer, &mut debug);

        debug.startup_finished();

        Self {
            state,
            compositor,
            renderer,
            viewport,
            debug,
            background_color,
            scale_factor,
            window_size,
            cursor_position,
            window_focused,
            mouse_interaction,
            resized: false,
            redraw_requested: true,
        }
    }

    /// The GUI update cycle called from the host.
    pub fn update(&mut self, delta_time: f64) {
        if self.state.is_queue_empty() {
            return;
        }

        // Executes all commands in the command queue and redraws the program.
        let command = self.state.update(
            Size::new(
                self.window_size.width as f32,
                self.window_size.height as f32,
            ),
            self.cursor_position,
            None, // The clipboard. We can try to implement this later.
            &mut self.renderer,
            &mut self.debug,
        );

        // If the application was updated
        if let Some(command) = command {
            // todo: store `command` to future queue

            let program = self.state.program();

            // Update subscriptions
            let subscription = program.subscription();
            // todo: track subscriptions

            // Update background color
            let background_color = program.background_color();
            // todo: update background color

            // Update scale factor
            let new_scale_factor = program.scale_factor();
            if self.scale_factor != new_scale_factor {
                self.scale_factor = new_scale_factor;

                self.viewport = Viewport::with_physical_size(self.window_size, self.scale_factor);

                // We relayout the UI with the new logical size.
                // The queue is empty, therefore this will never produce
                // a `Command`.
                //
                // TODO: Properly queue `WindowResized`
                let _ = self.state.update(
                    self.viewport.logical_size(),
                    self.cursor_position,
                    None, // clipboard
                    &mut self.renderer,
                    &mut self.debug,
                );
            }
        }

        self.redraw_requested = true;
    }

    /// The GUI render cycle called from the host.
    fn render(&mut self) {
        self.debug.render_started();

        if self.redraw_requested {
            self.redraw_requested = false;

            if self.resized {
                let physical_size = self.viewport.physical_size();

                /*
                self.swap_chain = self.compositor.create_swap_chain(
                    &self.surface,
                    physical_size.width,
                    physical_size.height,
                );
                */

                self.resized = false;
            }

            /*
            let new_mouse_interaction = self.compositor.draw(
                &mut self.renderer,
                &mut self.swap_chain,
                &self.viewport,
                self.background_color,
                self.state.primitive(),
                &self.debug.overlay(),
            );

            if new_mouse_interaction != self.mouse_interaction {
                window.set_cursor_icon(conversion::mouse_interaction(
                    new_mouse_interaction,
                ));

                self.mouse_interaction = new_mouse_interaction;
            }
            */

            // TODO: Handle animations!
            // Maybe we can use `ControlFlow::WaitUntil` for this.
        }

        self.debug.render_finished();
    }

    /// Called when the host resizes the window.
    pub fn resize(&mut self, width: u32, height: u32) {
        if self.window_size.width != width || self.window_size.height != height {
            self.resized = true;
            self.window_size.width = width;
            self.window_size.height = height;

            self.viewport = Viewport::with_physical_size(self.window_size, self.scale_factor);
        }
    }

    /// Called when the host wants to close the window.
    pub fn request_close(&mut self) {}

    /// Called when the cursor has moved in the window.
    pub fn cursor_moved(&mut self, x: f32, y: f32) {
        self.cursor_position.x = x;
        self.cursor_position.y = y;
    }

    /// Called when the window changes focus.
    pub fn window_focus(&mut self, focused: bool) {
        self.window_focused = focused;
    }

    /*
    /// Called when keyboard modifiers have changed.
    pub fn keyboard_modifiers(&mut self, new_modifiers) {

    }
    */

    /*
    /// Called on keyboard input.
    pub fn keyboard_input(&mut self, keyboard_input) {

    }
    */
}
