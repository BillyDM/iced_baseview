//! Configure your application.
use std::{borrow::Cow, fmt::Debug};

use baseview::{Size, WindowOpenOptions, WindowScalePolicy};

pub use crate::graphics::Settings as GraphicsSettings;

/// The settings of an application.
pub struct Settings {
    // /// The identifier of the application.
    // ///
    // /// If provided, this identifier may be used to identify the application or
    // /// communicate with it through the windowing system.
    // pub id: Option<String>,
    /// The [`Window`] settings.
    pub window: WindowOpenOptions,

    /// iced_baseview settings
    pub iced_baseview: IcedBaseviewSettings,

    /// The graphics settings.
    pub graphics_settings: GraphicsSettings,

    /// The fonts to load on boot.
    pub fonts: Vec<Cow<'static, [u8]>>,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            window: WindowOpenOptions {
                title: String::from("iced_baseview"),
                size: Size::new(500.0, 300.0),
                scale: WindowScalePolicy::SystemScaleFactor,
            },
            iced_baseview: IcedBaseviewSettings::default(),
            graphics_settings: GraphicsSettings::default(),
            fonts: Default::default(),
        }
    }
}

/// Any settings specific to `iced_baseview`.
#[derive(Debug, Clone, Copy, Default)]
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
