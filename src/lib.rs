// #![deny(missing_docs)]  // annoying while developing
#![deny(missing_debug_implementations)]
#![deny(unused_results)]
#![forbid(unsafe_code)]
#![forbid(rust_2018_idioms)]

#[doc(no_inline)]
pub use iced_native::*;

mod executor;

pub mod application;
pub mod settings;

pub use application::Application;
pub use settings::Settings;

pub use iced_graphics::Viewport;

/// The settings of the plugin host
#[derive(Debug, Copy, Clone)]
pub struct HostSettings {
    pub sample_rate: f64,
    pub buffer_size: f64,

    // whatever else is needed
    
}