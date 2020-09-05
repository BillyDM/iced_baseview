// #![deny(missing_docs)]  // annoying while developing
// #![deny(missing_debug_implementations)]
#![deny(unused_results)]
// #![forbid(unsafe_code)]
#![forbid(rust_2018_idioms)]

mod executor;

mod application;
pub mod settings;
pub use application::Application;
pub use executor::Executor;
pub use settings::Settings;
