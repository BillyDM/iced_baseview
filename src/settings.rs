//! Configure your application

/// The settings of an application.
#[derive(Debug, Clone, Default)]
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
#[derive(Debug, Clone)]
pub struct Window {
    /// The size of the window.
    pub size: (u32, u32),
}

impl Default for Window {
    fn default() -> Window {
        Window { size: (1024, 768) }
    }
}
