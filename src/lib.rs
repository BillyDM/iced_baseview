// #![deny(missing_docs)]  // annoying while developing
// #![deny(missing_debug_implementations)]
#![deny(unused_results)]
// #![forbid(unsafe_code)]
#![forbid(rust_2018_idioms)]

mod conversion;
mod handler;

mod application;
pub mod settings;
pub use application::Application;
pub use handler::Handler;
pub use settings::Settings;
