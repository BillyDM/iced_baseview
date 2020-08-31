//! Configure your application.

#[cfg(target_os = "windows")]
#[path = "settings/windows.rs"]
mod platform;
#[cfg(not(target_os = "windows"))]
#[path = "settings/not_windows.rs"]
mod platform;

pub use platform::PlatformSpecific;

/// The settings of a plugin application.
#[derive(Debug, Clone, Default)]
pub struct Settings<Flags> {
    /// The [`Window`] settings
    ///
    /// [`Window`]: struct.Window.html
    pub window: Window,

    /// The plugin-specific [`PluginSettings`] settings.
    ///
    /// [`PluginSettings`]: struct.PluginSettings.html
    pub plugin: PluginSettings,

    /// The data needed to initialize the user's [`PluginGUI`].
    ///
    /// This is where the user passes any initial user-defined
    /// data to their plugin.
    ///
    /// [`PluginGUI`]: trait.PluginGUI.html
    pub flags: Flags,
}

/// The window settings of a plugin application.
#[derive(Debug, Clone)]
pub struct Window {
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
    fn default() -> Self {
        Self {
            size: (500, 300),
            min_size: None,
            max_size: None,
            resizable: false,
        }
    }
}

/// The plugin-specific settings of a plugin application.
#[derive(Debug, Clone)]
pub struct PluginSettings {
    /// The name of the plugin.
    pub name: String,
    // Whatever else we need here
}

impl Default for PluginSettings {
    fn default() -> Self {
        Self {
            name: String::from("Iced baseview plugin"),
        }
    }
}
