//! Configure your application
use crate::WindowScalePolicy;

/// The settings of an application.
#[derive(Debug, Default)]
pub struct Settings<Flags> {
    /// The [`Window`] settings
    ///
    /// [`Window`]: struct.Window.html
    pub window: Window,

    /// The data needed to initialize an [`Application`].
    ///
    /// [`Application`]: trait.Application.html
    pub flags: Flags,
}

/// The window settings of an application.
#[derive(Debug)]
pub struct Window {
    /// The logical size of the window.
    pub logical_size: (u32, u32),
    /// The dpi scaling policy
    pub scale: WindowScalePolicy,
}

impl Default for Window {
    fn default() -> Window {
        Window {
            logical_size: (1024, 768),
            scale: WindowScalePolicy::SystemScaleFactor,
        }
    }
}
