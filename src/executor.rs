use crate::{Application, Settings};

use std::sync::mpsc;

use baseview::{Event, WindowInfo};
use iced_native::{program, Command, Debug, Element, Point, Size};
use iced_wgpu::{wgpu, Backend, Renderer, Viewport};

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

struct State<A: Application + 'static> {
    iced_state: program::State<IcedProgram<A>>,
    initial_command: Command<A::Message>,
    cursor_position: Point,
    debug: Debug,
    renderer: Renderer,
    viewport: Viewport,
    wgpu_instance: wgpu::Instance,
    surface: wgpu::Surface,
    device: wgpu::Device,
    queue: wgpu::Queue,
    swap_chain: wgpu::SwapChain,
    staging_belt: wgpu::util::StagingBelt,
    resized: bool,
}

pub struct Executor<A: Application + 'static> {
    flags: A::Flags,
    state: Option<State<A>>,
}

impl<A: Application + 'static> Executor<A> {
    pub fn run(settings: Settings, flags: A::Flags) {
        let window_open_options = baseview::WindowOpenOptions {
            title: settings.window.title.as_str(),
            width: settings.window.size.0 as usize,
            height: settings.window.size.1 as usize,
            parent: baseview::Parent::None,
        };

        let executor = Self { flags, state: None };

        // Create channel for sending messages from audio to GUI.
        let (_app_message_tx, app_message_rx) =
            mpsc::channel::<A::AudioToGuiMessage>();

        // Run the baseview window with the executor.
        let _ = baseview::Window::open(
            window_open_options,
            executor,
            app_message_rx,
        );
    }
}

impl<A: Application + 'static> baseview::AppWindow for Executor<A> {
    type AppMessage = A::AudioToGuiMessage;

    fn create_context(
        &mut self,
        window: baseview::RawWindow,
        window_info: &WindowInfo,
    ) {
        let window_size =
            Size::new(window_info.width as u32, window_info.height as u32);

        let viewport =
            Viewport::with_physical_size(window_size, window_info.scale);

        let wgpu_instance = wgpu::Instance::new(wgpu::BackendBit::PRIMARY);

        let surface = unsafe { wgpu_instance.create_surface(&window) };

        let (mut device, queue) = futures::executor::block_on(async {
            let adapter = wgpu_instance
                .request_adapter(&wgpu::RequestAdapterOptions {
                    power_preference: wgpu::PowerPreference::Default,
                    compatible_surface: Some(&surface),
                })
                .await
                .expect("Request adapter");

            adapter
                .request_device(
                    &wgpu::DeviceDescriptor {
                        features: wgpu::Features::empty(),
                        limits: wgpu::Limits::default(),
                        shader_validation: false,
                    },
                    None,
                )
                .await
                .expect("Request device")
        });

        let format = wgpu::TextureFormat::Bgra8UnormSrgb;

        let swap_chain = {
            device.create_swap_chain(
                &surface,
                &wgpu::SwapChainDescriptor {
                    usage: wgpu::TextureUsage::OUTPUT_ATTACHMENT,
                    format,
                    width: window_size.width,
                    height: window_size.height,
                    present_mode: wgpu::PresentMode::Mailbox,
                },
            )
        };

        // Initialize staging belt
        let staging_belt = wgpu::util::StagingBelt::new(5 * 1024);

        // Initialize iced
        let mut debug = Debug::new();
        let mut renderer = Renderer::new(Backend::new(
            &mut device,
            iced_wgpu::Settings::default(),
        ));

        // Initialize user program
        let (user_program, initial_command) = A::new(&self.flags);

        let iced_program = IcedProgram {
            user_app: user_program,
        };
        let iced_state = program::State::new(
            iced_program,
            viewport.logical_size(),
            Point::new(-1.0, -1.0),
            &mut renderer,
            &mut debug,
        );

        self.state = Some(State {
            iced_state,
            initial_command,
            cursor_position: Point::new(-1.0, -1.0),
            debug,
            renderer,
            viewport,
            wgpu_instance,
            surface,
            device,
            queue,
            swap_chain,
            staging_belt,
            resized: false,
        });
    }

    fn draw(&mut self) {}

    fn on_event(&mut self, event: Event) {
        match event {
            Event::CursorMotion(x, y) => {
                println!("Cursor moved, x: {}, y: {}", x, y);
            }
            Event::MouseDown(button_id) => {
                println!("Mouse down, button id: {:?}", button_id);
            }
            Event::MouseUp(button_id) => {
                println!("Mouse up, button id: {:?}", button_id);
            }
            Event::MouseScroll(mouse_scroll) => {
                println!("Mouse scroll, {:?}", mouse_scroll);
            }
            Event::MouseClick(mouse_click) => {
                println!("Mouse click, {:?}", mouse_click);
            }
            Event::KeyDown(keycode) => {
                println!("Key down, keycode: {}", keycode);
            }
            Event::KeyUp(keycode) => {
                println!("Key up, keycode: {}", keycode);
            }
            Event::CharacterInput(char_code) => {
                println!("Character input, char_code: {}", char_code);
            }
            Event::WindowResized(window_info) => {
                println!("Window resized, {:?}", window_info);
            }
            Event::WindowFocus => {
                println!("Window focused");
            }
            Event::WindowUnfocus => {
                println!("Window unfocused");
            }
            Event::WillClose => {
                println!("Window will close");
            }
        }
    }

    fn on_app_message(&mut self, message: Self::AppMessage) {}
}
