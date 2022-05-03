//! Configure your application;

use baseview::WindowOpenOptions;

/// Any settings specific to `iced_baseview`.
#[derive(Debug, Clone, Copy)]
pub struct IcedBaseviewSettings {
    /// Ignore key inputs, except for modifier keys such as SHIFT and ALT
    pub ignore_non_modifier_keys: bool,

    /// Always redraw whenever the baseview window updates instead of only when iced wants to update
    /// the window. This works around a current baseview limitation where it does not support
    /// trigger a redraw on window visibility change (which may cause blank windows when opening or
    /// reopening the editor) and an iced limitation where it's not possible to have animations
    /// without using an asynchronous timer stream to send redraw messages to the application.
    pub always_redraw: bool,
}

/// The settings of an application.
#[allow(missing_debug_implementations)]
pub struct Settings<Flags> {
    /// The [`Window`] settings
    ///
    /// [`Window`]: struct.Window.html
    pub window: WindowOpenOptions,

    /// Special settings for controlling the behavior of `iced_baseview`.
    pub iced_baseview: IcedBaseviewSettings,

    /// The data needed to initialize an [`Application`].
    ///
    /// [`Application`]: trait.Application.html
    pub flags: Flags,
}
