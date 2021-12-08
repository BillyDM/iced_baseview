//! Choose your preferred executor to power your application.
pub use iced_futures::{executor::Null, Executor};

pub use platform::Default;

mod platform {
    use iced_futures::{executor, futures};

    #[cfg(feature = "with-tokio")]
    type Executor = executor::Tokio;

    #[cfg(all(not(feature = "with-tokio"), feature = "with-async-std"))]
    type Executor = executor::AsyncStd;

    #[cfg(not(any(feature = "with-tokio", feature = "with-async-std")))]
    type Executor = executor::ThreadPool;

    /// A default cross-platform executor.
    ///
    /// - On native platforms, it will use:
    ///   - `iced_futures::executor::Tokio` when the `tokio` feature is enabled.
    ///   - `iced_futures::executor::AsyncStd` when the `async-std` feature is
    ///     enabled.
    ///   - `iced_futures::executor::ThreadPool` otherwise.
    #[derive(Debug)]
    pub struct Default(Executor);

    impl super::Executor for Default {
        fn new() -> Result<Self, futures::io::Error> {
            Ok(Default(Executor::new()?))
        }

        fn spawn(
            &self,
            future: impl futures::Future<Output = ()> + Send + 'static,
        ) {
            let _ = self.0.spawn(future);
        }

        fn enter<R>(&self, f: impl FnOnce() -> R) -> R {
            super::Executor::enter(&self.0, f)
        }
    }
}
