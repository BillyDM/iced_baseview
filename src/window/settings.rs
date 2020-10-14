/// The window settings of an application.
#[derive(Debug, Clone)]
pub struct Settings {
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

impl Default for Settings {
    fn default() -> Self {
        Self {
            title: String::from("iced_baseview"),
            size: (500, 300),
            min_size: None,
            max_size: None,
            resizable: true,
        }
    }
}