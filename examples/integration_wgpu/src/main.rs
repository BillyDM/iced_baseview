mod controls;
mod scene;

use controls::Controls;
use futures::task::SpawnExt;
use scene::Scene;

use baseview::{EventStatus, WindowHandler, WindowScalePolicy};
use iced_baseview::Size;
use iced_native::clipboard;
use iced_native::keyboard::Modifiers;
use iced_native::Event as IcedEvent;
use iced_native::{program, Debug};
use iced_wgpu::{wgpu, Backend, Renderer, Settings, Viewport};

struct WgpuIntegrationWindow {
    events: Vec<IcedEvent>,
    debug: Debug,

    cursor_position: iced_baseview::Point,
    resized: bool,
    logical_size: Size,
    modifiers: Modifiers,
    window_info: baseview::WindowInfo,

    viewport: Viewport,
    device: wgpu::Device,
    swap_chain: wgpu::SwapChain,
    surface: wgpu::Surface,
    format: wgpu::TextureFormat,
    staging_belt: wgpu::util::StagingBelt,
    queue: wgpu::Queue,

    local_pool: futures::executor::LocalPool,
    scene: Scene,
    renderer: Renderer,
    state: program::State<Controls>,
}

impl WgpuIntegrationWindow {
    fn render_window(&mut self) {
        if self.resized {
            let size = self.window_info.physical_size();

            self.swap_chain = self.device.create_swap_chain(
                &self.surface,
                &wgpu::SwapChainDescriptor {
                    usage: wgpu::TextureUsage::RENDER_ATTACHMENT,
                    format: self.format,
                    width: size.width,
                    height: size.height,
                    present_mode: wgpu::PresentMode::Mailbox,
                },
            );

            self.resized = false;
        }

        let frame = self.swap_chain.get_current_frame().expect("Next frame");

        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });

        let program = self.state.program();

        {
            // We clear the frame
            let mut render_pass = self.scene.clear(
                &frame.output.view,
                &mut encoder,
                conversion::iced_baseview_color_to_f64_array(program.background_color()),
            );

            // Draw the scene
            self.scene.draw(&mut render_pass);
        }

        // And then iced on top
        let _mouse_interaction = self.renderer.backend_mut().draw(
            &self.device,
            &mut self.staging_belt,
            &mut encoder,
            &frame.output.view,
            &self.viewport,
            self.state.primitive(),
            &self.debug.overlay(),
        );

        // Then we submit the work
        self.staging_belt.finish();
        self.queue.submit(Some(encoder.finish()));

        //TODO // Update the mouse cursor
        //TODO window.set_cursor_icon(iced_winit::conversion::mouse_interaction(mouse_interaction));

        // And recall staging buffers
        self.local_pool
            .spawner()
            .spawn(self.staging_belt.recall())
            .expect("Recall staging buffers");

        self.local_pool.run_until_stalled();
    }
}

impl WindowHandler for WgpuIntegrationWindow {
    fn on_frame(&mut self, _window: &mut baseview::Window) {
        self.render_window();
    }

    fn on_event(&mut self, _window: &mut baseview::Window, event: baseview::Event) -> EventStatus {
        match &event {
            baseview::Event::Mouse(e) => {
                if let baseview::MouseEvent::CursorMoved { position } = e {
                    self.cursor_position =
                        conversion::baseview_point_to_iced_baseview_point(position);
                }
            }
            baseview::Event::Keyboard(_) => {}
            baseview::Event::Window(e) => {
                match e {
                    baseview::WindowEvent::Resized(window_info) => {
                        self.logical_size = conversion::baseview_size_to_iced_baseview_size(
                            &window_info.logical_size(),
                        );
                        self.viewport = Viewport::with_physical_size(
                            Size::new(
                                window_info.physical_size().width,
                                window_info.physical_size().height,
                            ),
                            window_info.scale(),
                        );
                        self.window_info = *window_info;
                        self.resized = true;
                    }
                    baseview::WindowEvent::WillClose => {
                        // TODO: Handle window close events.
                    }
                    _ => {}
                }
            }
        }

        iced_baseview::conversion::baseview_to_iced_events(
            event,
            &mut self.events,
            &mut self.modifiers,
        );
        for event in self.events.drain(..) {
            self.state.queue_event(event);
        }
        if !self.state.is_queue_empty() {
            // We update iced
            let _ = self.state.update(
                self.viewport.logical_size(),
                self.cursor_position,
                &mut self.renderer,
                &mut clipboard::Null,
                &mut self.debug,
            );
        }
        EventStatus::Captured
    }
}

fn main() {
    env_logger::init();

    // Logical size.
    let size = baseview::Size::new(500.0, 300.0);

    let options = baseview::WindowOpenOptions {
        title: "baseview".into(),
        size,
        scale: WindowScalePolicy::SystemScaleFactor,
    };

    let scaling = match options.scale {
        WindowScalePolicy::ScaleFactor(scale) => scale,
        WindowScalePolicy::SystemScaleFactor => 1.0,
    };

    baseview::Window::open_blocking(options, move |window| {
        let window_info = baseview::WindowInfo::from_logical_size(size, scaling);

        let viewport = Viewport::with_physical_size(
            iced_baseview::Size::new(
                window_info.physical_size().width,
                window_info.physical_size().height,
            ),
            window_info.scale(),
        );

        // Initialize wgpu
        let instance = wgpu::Instance::new(wgpu::BackendBit::PRIMARY);
        let surface = unsafe { instance.create_surface(window) };

        let (device, queue) = futures::executor::block_on(async {
            let adapter = instance
                .request_adapter(&wgpu::RequestAdapterOptions {
                    power_preference: wgpu::PowerPreference::HighPerformance,
                    compatible_surface: Some(&surface),
                })
                .await
                .expect("Request adapter");

            adapter
                .request_device(
                    &wgpu::DeviceDescriptor {
                        label: None,
                        features: wgpu::Features::empty(),
                        limits: wgpu::Limits::default(),
                    },
                    None,
                )
                .await
                .expect("Request device")
        });

        let format = wgpu::TextureFormat::Bgra8UnormSrgb;

        let swap_chain = {
            let size = window_info.physical_size();

            device.create_swap_chain(
                &surface,
                &wgpu::SwapChainDescriptor {
                    usage: wgpu::TextureUsage::RENDER_ATTACHMENT,
                    format,
                    width: size.width,
                    height: size.height,
                    present_mode: wgpu::PresentMode::Mailbox,
                },
            )
        };

        // Initialize staging belt and local pool
        let staging_belt = wgpu::util::StagingBelt::new(5 * 1024);
        let local_pool = futures::executor::LocalPool::new();

        // Initialize scene and GUI controls
        let scene = Scene::new(&device);
        let controls = Controls::new();

        // Initialize iced
        let mut debug = Debug::new();
        let mut renderer = Renderer::new(Backend::new(&device, Settings::default()));

        let state = program::State::new(
            controls,
            viewport.logical_size(),
            iced_baseview::Point::new(-1.0, -1.0),
            &mut renderer,
            &mut debug,
        );

        WgpuIntegrationWindow {
            events: Vec::with_capacity(64),
            debug,

            cursor_position: iced_baseview::Point::new(-1.0, -1.0),
            resized: false,
            logical_size: iced_baseview::Size::new(
                window_info.logical_size().width as f32,
                window_info.logical_size().height as f32,
            ),
            modifiers: Modifiers::default(),
            window_info,

            viewport,
            device,
            surface,
            swap_chain,
            format,
            staging_belt,
            queue,

            local_pool,
            scene,
            renderer,
            state,
        }
    });
}

mod conversion {

    pub fn baseview_size_to_iced_baseview_size(size: &baseview::Size) -> iced_baseview::Size {
        iced_baseview::Size::new(size.width as f32, size.height as f32)
    }

    pub fn baseview_point_to_iced_baseview_point(point: &baseview::Point) -> iced_baseview::Point {
        iced_baseview::Point::new(point.x as f32, point.y as f32)
    }

    pub fn iced_baseview_color_to_f64_array(color: iced_baseview::Color) -> [f64; 4] {
        [
            color.r as f64,
            color.g as f64,
            color.b as f64,
            color.a as f64,
        ]
    }
}
