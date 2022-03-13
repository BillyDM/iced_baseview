//! Configure your application;

use baseview::WindowOpenOptions;

/// The settings of an application.
#[allow(missing_debug_implementations)]
pub struct Settings<Flags> {
    /// The [`Window`] settings
    ///
    /// [`Window`]: struct.Window.html
    pub window: WindowOpenOptions,

    /// Ignore key inputs, except for modifier keys such as SHIFT and ALT
    pub ignore_non_modifier_keys: bool,

    /// The data needed to initialize an [`Application`].
    ///
    /// [`Application`]: trait.Application.html
    pub flags: Flags,
}
