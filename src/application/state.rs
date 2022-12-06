use baseview::WindowScalePolicy;

use iced_graphics::Viewport;
use iced_native::{Color, Debug, Point, Size};

use crate::application::Application;
use crate::application::{self, StyleSheet as _};

use std::marker::PhantomData;

/// The state of a windowed [`Application`].
#[allow(missing_debug_implementations)]
pub struct State<A: Application>
where
    <A::Renderer as iced_native::Renderer>::Theme: application::StyleSheet,
{
    title: String,
    viewport: Viewport,
    viewport_version: usize,
    cursor_position: Point,
    theme: <A::Renderer as iced_native::Renderer>::Theme,
    appearance: application::Appearance,
    application: PhantomData<A>,

    system_scale_factor: f64,
    scale_policy: WindowScalePolicy,
    modifiers: iced_core::keyboard::Modifiers,
}

impl<A: Application> State<A>
where
    <A::Renderer as iced_native::Renderer>::Theme: application::StyleSheet,
{
    /// Creates a new [`State`] for the provided [`Application`] and window.
    pub fn new(application: &A, viewport: Viewport, scale_policy: WindowScalePolicy) -> Self {
        let title = application.title();
        let theme = application.theme();
        let appearance = theme.appearance(&application.style());

        Self {
            title,
            viewport,
            viewport_version: 0,
            // TODO: Encode cursor availability in the type-system
            cursor_position: Point::new(-1.0, -1.0),
            // modifiers: winit::event::ModifiersState::default(),
            theme,
            appearance,
            application: PhantomData,

            system_scale_factor: 1.0,
            scale_policy,
            modifiers: Default::default(),
        }
    }

    /// Returns the current [`Viewport`] of the [`State`].
    pub fn viewport(&self) -> &Viewport {
        &self.viewport
    }

    /// Returns the version of the [`Viewport`] of the [`State`].
    ///
    /// The version is incremented every time the [`Viewport`] changes.
    pub fn viewport_version(&self) -> usize {
        self.viewport_version
    }

    /// Returns the physical [`Size`] of the [`Viewport`] of the [`State`].
    pub fn physical_size(&self) -> Size<u32> {
        self.viewport.physical_size()
    }

    /// Returns the logical [`Size`] of the [`Viewport`] of the [`State`].
    pub fn logical_size(&self) -> Size<f32> {
        self.viewport.logical_size()
    }

    /// Returns the current cursor position of the [`State`].
    pub fn cursor_position(&self) -> Point {
        self.cursor_position
    }

    // /// Returns the current keyboard modifiers of the [`State`].
    // pub fn modifiers(&self) -> winit::event::ModifiersState {
    //     self.modifiers
    // }

    /// Returns the current theme of the [`State`].
    pub fn theme(&self) -> &<A::Renderer as iced_native::Renderer>::Theme {
        &self.theme
    }

    /// Returns the current background [`Color`] of the [`State`].
    pub fn background_color(&self) -> Color {
        self.appearance.background_color
    }

    /// Returns the current text [`Color`] of the [`State`].
    pub fn text_color(&self) -> Color {
        self.appearance.text_color
    }

    /// Processes the provided window event and updates the [`State`]
    /// accordingly.
    ///
    /// Does **not** update modifiers.
    pub fn update(&mut self, event: &baseview::Event, _debug: &mut Debug) {
        match event {
            baseview::Event::Window(baseview::WindowEvent::Resized(window_info)) => {
                // Cache system window info in case users changes their scale policy in the future.
                self.system_scale_factor = window_info.scale();

                let scale = match self.scale_policy {
                    WindowScalePolicy::ScaleFactor(scale) => scale,
                    WindowScalePolicy::SystemScaleFactor => self.system_scale_factor,
                };

                self.viewport = Viewport::with_physical_size(
                    Size::new(
                        window_info.physical_size().width,
                        window_info.physical_size().height,
                    ),
                    scale,
                );

                self.viewport_version = self.viewport_version.wrapping_add(1);
            }
            baseview::Event::Mouse(baseview::MouseEvent::CursorMoved {
                position,
                modifiers: _,
            }) => {
                self.cursor_position.x = position.x as f32;
                self.cursor_position.y = position.y as f32;

                // TODO: Encode cursor moving outside of the window.
            }
            baseview::Event::Keyboard(_) => {
                #[cfg(feature = "debug")]
                {
                    use keyboard_types::{Key, KeyState};
                    if event.key == Key::F12 && event.state == KeyState::Down {
                        _debug.toggle();
                    }
                }
            }
            /*
            WindowEvent::ScaleFactorChanged {
                scale_factor: new_scale_factor,
                new_inner_size,
            } => {
                let size =
                    Size::new(new_inner_size.width, new_inner_size.height);

                self.viewport = Viewport::with_physical_size(
                    size,
                    new_scale_factor * self.scale_factor,
                );

                self.viewport_version = self.viewport_version.wrapping_add(1);
            }
            WindowEvent::CursorMoved { position, .. }
            | WindowEvent::Touch(Touch {
                location: position, ..
            }) => {
                self.cursor_position = *position;
            }
            WindowEvent::CursorLeft { .. } => {
                // TODO: Encode cursor availability in the type-system
                self.cursor_position =
                    winit::dpi::PhysicalPosition::new(-1.0, -1.0);
            }
            WindowEvent::ModifiersChanged(new_modifiers) => {
                self.modifiers = *new_modifiers;
            }
            #[cfg(feature = "debug")]
            WindowEvent::KeyboardInput {
                input:
                    winit::event::KeyboardInput {
                        virtual_keycode: Some(winit::event::VirtualKeyCode::F12),
                        state: winit::event::ElementState::Pressed,
                        ..
                    },
                ..
            } => _debug.toggle(),
            */
            _ => {}
        }
    }

    /// Synchronizes the [`State`] with its [`Application`] and its respective
    /// window.
    ///
    /// Normally an [`Application`] should be synchronized with its [`State`]
    /// and window after calling [`Application::update`].
    ///
    /// [`Application::update`]: crate::Program::update
    pub fn synchronize(&mut self, application: &A) {
        // Update window title
        let new_title = application.title();

        if self.title != new_title {
            // window.set_title(&new_title); // TODO?

            self.title = new_title;
        }

        // Update scale policy
        let new_scale_policy = application.scale_policy();

        match &new_scale_policy {
            WindowScalePolicy::SystemScaleFactor => match &self.scale_policy {
                WindowScalePolicy::SystemScaleFactor => {}
                WindowScalePolicy::ScaleFactor(_) => {
                    self.scale_policy = WindowScalePolicy::SystemScaleFactor;

                    self.viewport = Viewport::with_physical_size(
                        self.viewport.physical_size(),
                        self.system_scale_factor,
                    );

                    self.viewport_version = self.viewport_version.wrapping_add(1);
                }
            },
            WindowScalePolicy::ScaleFactor(new_scale) => {
                let matches = match &self.scale_policy {
                    WindowScalePolicy::SystemScaleFactor => false,
                    WindowScalePolicy::ScaleFactor(scale) => {
                        (*scale - *new_scale).abs() < f64::EPSILON
                    }
                };

                if !matches {
                    self.scale_policy = WindowScalePolicy::ScaleFactor(*new_scale);

                    self.viewport =
                        Viewport::with_physical_size(self.viewport.physical_size(), *new_scale);

                    self.viewport_version = self.viewport_version.wrapping_add(1);
                }
            }
        }

        // Update theme and appearance
        self.theme = application.theme();
        self.appearance = self.theme.appearance(&application.style());
    }

    pub(crate) fn modifiers_mut(&mut self) -> &mut iced_core::keyboard::Modifiers {
        &mut self.modifiers
    }
}
