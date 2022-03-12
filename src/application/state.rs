use iced_graphics::Viewport;
use iced_native::keyboard::Modifiers as IcedModifiers;
use iced_native::Debug;
use keyboard_types::Modifiers;
use std::marker::PhantomData;

use crate::{Application, Color, Point, Size};

use baseview::WindowScalePolicy;

/// The state of a windowed [`Application`].
#[derive(Debug)]
pub struct State<A: Application + Send> {
    background_color: Color,
    scale_policy: WindowScalePolicy,
    system_scale_factor: f64,
    viewport: Viewport,
    viewport_version: usize,
    cursor_position: Point,
    modifiers: IcedModifiers,
    application: PhantomData<A>,
}

impl<A: Application + Send> State<A> {
    /// Creates a new [`State`] for the provided [`Application`] and window.
    pub fn new(
        application: &A,
        viewport: Viewport,
        scale_policy: WindowScalePolicy,
    ) -> Self {
        //let mode = application.mode();
        let background_color = application.background_color();
        //let scale_factor = application.scale_factor();

        Self {
            background_color,
            scale_policy,
            system_scale_factor: 1.0,
            viewport,
            viewport_version: 0,
            // TODO: Encode cursor availability in the type-system
            cursor_position: Point::new(-1.0, -1.0),
            modifiers: IcedModifiers::default(),
            application: PhantomData,
        }
    }

    /// Returns the current background [`Color`] of the [`State`].
    pub fn background_color(&self) -> Color {
        self.background_color
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

    /*
    /// Returns the current scale factor of the [`Viewport`] of the [`State`].
    pub fn scale_factor(&self) -> f64 {
        self.viewport.scale_factor()
    }
    */

    /// Returns the current cursor position of the [`State`].
    pub fn cursor_position(&self) -> Point {
        self.cursor_position
    }

    /*
    /// Returns the current keyboard modifiers of the [`State`].
    pub fn modifiers(&self) -> keyboard::Modifiers {
        self.modifiers
    }
    */

    /// Processes the provided window event and updates the [`State`]
    /// accordingly.
    pub fn update(&mut self, event: &baseview::Event, _debug: &mut Debug) {
        match event {
            baseview::Event::Window(baseview::WindowEvent::Resized(
                window_info,
            )) => {
                // Cache system window info in case users changes their scale policy in the future.
                self.system_scale_factor = window_info.scale();

                let scale = match self.scale_policy {
                    WindowScalePolicy::ScaleFactor(scale) => scale,
                    WindowScalePolicy::SystemScaleFactor => {
                        self.system_scale_factor
                    }
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
                modifiers,
            }) => {
                self.update_modifiers(*modifiers);

                self.cursor_position.x = position.x as f32;
                self.cursor_position.y = position.y as f32;

                // TODO: Encode cursor moving outside of the window.
            }
            baseview::Event::Mouse(
                baseview::MouseEvent::ButtonPressed { modifiers, .. }
                | baseview::MouseEvent::ButtonReleased { modifiers, .. }
                | baseview::MouseEvent::WheelScrolled { modifiers, .. },
            ) => {
                self.update_modifiers(*modifiers);
            }
            baseview::Event::Keyboard(event) => {
                self.update_modifiers(event.modifiers);

                #[cfg(feature = "debug")]
                {
                    use keyboard_types::{Key, KeyState};
                    if event.key == Key::F12 && event.state == KeyState::Down {
                        _debug.toggle();
                    }
                }
            }
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
        /*
        // Update window mode
        let new_mode = application.mode();

        if self.mode != new_mode {
            window.set_fullscreen(conversion::fullscreen(
                window.current_monitor(),
                new_mode,
            ));

            self.mode = new_mode;
        }
        */

        // Update background color
        self.background_color = application.background_color();

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

                    self.viewport_version =
                        self.viewport_version.wrapping_add(1);
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
                    self.scale_policy =
                        WindowScalePolicy::ScaleFactor(*new_scale);

                    self.viewport = Viewport::with_physical_size(
                        self.viewport.physical_size(),
                        *new_scale,
                    );

                    self.viewport_version =
                        self.viewport_version.wrapping_add(1);
                }
            }
        }
    }

    fn update_modifiers(&mut self, modifiers: Modifiers) {
        self.modifiers
            .set(IcedModifiers::SHIFT, modifiers.contains(Modifiers::SHIFT));
        self.modifiers
            .set(IcedModifiers::CTRL, modifiers.contains(Modifiers::CONTROL));
        self.modifiers
            .set(IcedModifiers::ALT, modifiers.contains(Modifiers::ALT));
        self.modifiers
            .set(IcedModifiers::LOGO, modifiers.contains(Modifiers::META));
    }
}
