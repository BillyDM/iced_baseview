use crate::{Application, Settings};

use baseview::{
    Event, KeyboardEvent, MouseEvent, Window, WindowEvent, WindowHandle,
    WindowHandler,
};
use iced_graphics::Viewport;
use iced_native::{program, Color, Command, Debug, Element, Point, Size};
use raw_window_handle::RawWindowHandle;
//use iced_wgpu::{wgpu, Backend, Renderer, Viewport};
//use futures::task::SpawnExt;

#[cfg(feature = "wgpu")]
type Renderer = iced_wgpu::Renderer;

#[cfg(feature = "wgpu")]
type Compositor = iced_wgpu::window::Compositor;

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
}

impl<A: Application + 'static> Handler<A> {
    pub fn open(
        settings: Settings,
        parent: Option<RawWindowHandle>,
    ) -> WindowHandle {
        let baseview_parent = if let Some(parent) = parent {
            baseview::Parent::WithParent(parent)
        } else {
            baseview::Parent::None
        };

        let window_open_options = baseview::WindowOpenOptions {
            title: settings.window.title,
            width: settings.window.size.0 as usize,
            height: settings.window.size.1 as usize,
            parent: baseview_parent,
        };

        Window::open::<Handler<A>>(window_open_options)
    }
}

impl<A: Application + 'static> WindowHandler for Handler<A> {
    type Message = A::AudioToGuiMessage;

    fn build(window: &mut Window) -> Self {
        use iced_graphics::window::Compositor as IGCompositor;

        let window_info = window.window_info();

        let mut debug = Debug::new();

        let window_size =
            Size::new(window_info.width as u32, window_info.height as u32);

        let viewport =
            Viewport::with_physical_size(window_size, window_info.scale);

        let compositor_settings = A::compositor_settings();

        let (mut compositor, mut renderer) =
            <Compositor as IGCompositor>::new(compositor_settings).unwrap();

        let surface = compositor.create_surface(window);

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
        }
    }

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

    fn on_event(&mut self, _window: &mut Window, event: Event) {
        use iced_graphics::window::Compositor as IGCompositor;

        if let Event::Mouse(MouseEvent::CursorMoved { x, y }) = event {
            self.cursor_position.x = x as f32;
            self.cursor_position.y = y as f32;
        };

        if let Event::Window(WindowEvent::Resized(window_info)) = event {
            self.window_size.width = window_info.width;
            self.window_size.height = window_info.height;

            self.viewport = Viewport::with_physical_size(
                self.window_size,
                self.scale_factor,
            );

            let physical_size = self.viewport.physical_size();

            self.swap_chain = self.compositor.create_swap_chain(
                &self.surface,
                physical_size.width,
                physical_size.height,
            );
        }

        if let Some(iced_event) =
            crate::conversion::baseview_to_iced_event(event)
        {
            self.iced_state.queue_event(iced_event);

            self.redraw_requested = true;
        }
    }

    fn on_message(&mut self, _window: &mut Window, _message: Self::Message) {}
}
