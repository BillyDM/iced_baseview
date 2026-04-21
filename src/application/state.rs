use baseview::WindowScalePolicy;

use crate::application::{Appearance, Application, DefaultStyle};
use crate::core::mouse;
use crate::core::{Color, Size};
use crate::graphics::Viewport;

use std::marker::PhantomData;

/// The state of a windowed [`Application`].
#[allow(missing_debug_implementations)]
pub struct State<A: Application>
where
    A::Theme: DefaultStyle,
{
    title: String,
    viewport: Viewport,
    viewport_version: usize,
    cursor_position: Option<iced_runtime::core::Point>,
    theme: A::Theme,
    appearance: Appearance,
    application: PhantomData<A>,

    system_scale_factor: f64,
    scale_policy: WindowScalePolicy,
    modifiers: iced_runtime::core::keyboard::Modifiers,

    #[cfg(feature = "toggle_debug")]
    debug_enabled: bool,
}

impl<A: Application> State<A>
where
    A::Theme: DefaultStyle,
{
    /// Creates a new [`State`] for the provided [`Application`] and window.
    pub fn new(application: &A, viewport: Viewport) -> Self {
        let title = application.title();
        let theme = application.theme();
        let appearance = application.style(&theme);
        let scale_policy = application.scale_policy();

        Self {
            title,
            viewport,
            viewport_version: 0,
            cursor_position: None,
            theme,
            appearance,
            application: PhantomData,

            system_scale_factor: 1.0,
            scale_policy,
            modifiers: Default::default(),
            #[cfg(feature = "toggle_debug")]
            debug_enabled: false,
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
    pub fn cursor(&self) -> mouse::Cursor {
        self.cursor_position
            .map(mouse::Cursor::Available)
            .unwrap_or(mouse::Cursor::Unavailable)
    }

    /// Returns the current theme of the [`State`].
    pub fn theme(&self) -> &A::Theme {
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
    pub fn update(&mut self, event: &baseview::Event) {
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
                self.cursor_position = Some(crate::core::Point {
                    x: position.x as f32,
                    y: position.y as f32,
                });

                // TODO: Encode cursor moving outside of the window.
            }
            #[allow(unused_variables)]
            baseview::Event::Keyboard(event) => {
                #[cfg(feature = "toggle_debug")]
                {
                    use keyboard_types::{Key, KeyState};
                    if event.key == Key::F12 && event.state == KeyState::Down {
                        if self.debug_enabled {
                            iced_debug::enable();
                            self.debug_enabled = true;
                        } else {
                            iced_debug::disable();
                            self.debug_enabled = true;
                        }
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
    /// [`Application::update`]: crate::Application::update
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
        self.appearance = application.style(&self.theme);
    }

    pub(crate) fn modifiers_mut(&mut self) -> &mut iced_runtime::core::keyboard::Modifiers {
        &mut self.modifiers
    }
}
