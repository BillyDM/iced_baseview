//! Configure your application
use crate::window;

/// The settings of an application.
#[derive(Debug, Clone, Default)]
pub struct Settings {
    /// The [`Window`] settings
    ///
    /// [`Window`]: struct.Window.html
    pub window: window::Settings,
}
