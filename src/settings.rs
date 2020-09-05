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
    /// The title of the window.
    pub title: String,

    /// The size of the window.
    pub size: (u32, u32),

    /// The minimum size of the window.
    pub min_size: Option<(u32, u32)>,

    /// The maximum size of the window.
    pub max_size: Option<(u32, u32)>,

    /// Whether the window should be resizable or not.
    pub resizable: bool,
}

impl Default for Window {
    fn default() -> Window {
        Window {
            title: String::from("iced_baseview"),
            size: (500, 300),
            min_size: None,
            max_size: None,
            resizable: true,
        }
    }
}
