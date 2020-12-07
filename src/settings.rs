//! Configure your application;

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum WindowScalePolicy {
    /// Use the provided scale factor.
    ScaleFactor(f64),
    /// Use the system's scale factor.
    SystemScaleFactor,
}

impl Default for WindowScalePolicy {
    fn default() -> Self {
        WindowScalePolicy::SystemScaleFactor
    }
}

impl From<WindowScalePolicy> for baseview::WindowScalePolicy {
    fn from(p: WindowScalePolicy) -> Self {
        match p {
            WindowScalePolicy::ScaleFactor(scale) => {
                baseview::WindowScalePolicy::ScaleFactor(scale)
            }
            WindowScalePolicy::SystemScaleFactor => {
                baseview::WindowScalePolicy::SystemScaleFactor
            }
        }
    }
}

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
    /// The initial dpi scaling policy
    pub scale_policy: WindowScalePolicy,
}

impl Default for Window {
    fn default() -> Window {
        Window {
            logical_size: (1024, 768),
            scale_policy: Default::default(),
        }
    }
}
