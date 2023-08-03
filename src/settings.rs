
//! Configure your application.
use std::fmt::Debug;

use baseview::WindowOpenOptions;

/// The settings of an application.
pub struct Settings<Flags> {
    // /// The identifier of the application.
    // ///
    // /// If provided, this identifier may be used to identify the application or
    // /// communicate with it through the windowing system.
    // pub id: Option<String>,
    /// The [`Window`] settings.
    pub window: WindowOpenOptions,

    /// The data needed to initialize an [`Application`].
    ///
    /// [`Application`]: crate::Application
    pub flags: Flags,

    /// iced_baseview settings
    pub iced_baseview: IcedBaseviewSettings,
}

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
