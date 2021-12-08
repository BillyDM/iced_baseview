//! Configure your application;

use baseview::WindowOpenOptions;

/// The settings of an application.
#[allow(missing_debug_implementations)]
pub struct Settings<Flags> {
    /// The [`Window`] settings
    ///
    /// [`Window`]: struct.Window.html
    pub window: WindowOpenOptions,

    /// Override antialiasing settings and use maximum supported number of
    /// samples. Only available with feature `with-glow`.
    #[cfg(feature = "with-glow")]
    pub use_max_aa_samples: bool,

    /// The data needed to initialize an [`Application`].
    ///
    /// [`Application`]: trait.Application.html
    pub flags: Flags,
}
