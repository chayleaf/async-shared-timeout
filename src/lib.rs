#![cfg_attr(all(not(feature = "std"), not(feature = "async-io"), not(feature = "tokio")), no_std)]
#![cfg_attr(docsrs, feature(doc_cfg))]

//! A crate that offers a way to create a timeout that can be reset and shared.
//! Additionally, stream timeout is offered under a feature flag.
//!
//! # Feature flags:
//! 
//! **Wrapper**
//!
//! - `wrapper` - enable a wrapper around types that you can use for easier resetting. By default,
//!               only future support is enabled (reset the timer upon future completion).
//! - `read-write` - enable async `Read`/`Write` trait support for the wrapper (reset the timer
//!                  upon successful read/write operations)
//! - `stream` - enable `Stream` support for the wrapper (reset the timer upon stream advancement).
//!
//! **Integration with other runtimes**
//!
//! - `std` (enabled by default) - enable `std` integration. Currently it's only used to enable
//!                                `Arc` and `AsRawFd` support for the wrapper.
//! - `tokio` (enabled by default) - [`tokio`](https://docs.rs/tokio) support
//! - `async-io` - support [`async-io`](https://docs.rs/async-io) as the timer runtime.
//! - `futures-io` - support [`futures-io`](https://docs.rs/futures-io) traits.
//! - `async-std` - [`async-std`](https://docs.rs/async-std) support (enables `async-io` and `futures-io`).
//!
//! See struct documentation for examples.
use core::{future::Future, pin::Pin, sync::atomic::Ordering, task::{Context, Poll}, time::Duration};
use portable_atomic::AtomicU64;

pub mod runtime;
use runtime::{Instant, Runtime, Sleep};

/// A shared timeout.
///
/// # Example
///
/// ```
/// # async fn read_command() -> Option<&'static str> { Some("command") }
/// # async fn example_fn() {
/// use std::time::Duration;
///
/// let timeout_secs = Duration::from_secs(10);
/// // Use the tokio runtime
/// let runtime = async_shared_timeout::runtime::Tokio::new();
/// let timeout = async_shared_timeout::Timeout::new(runtime, timeout_secs);
/// tokio::select! {
///     _ = timeout.wait() => {
///         println!("timeout expired!");
///     }
///     _ = async {
///         while let Some(cmd) = read_command().await {
///             println!("command received: {:?}", cmd);
///             timeout.reset();
///         }
///     } => {
///         println!("no more commands!");
///     }
/// }
/// # }
/// ```
#[derive(Debug)]
pub struct Timeout<R: Runtime> {
    runtime: R,
    epoch: R::Instant,
    timeout_from_epoch_ns: AtomicU64,
    default_timeout: AtomicU64,
}

impl<R: Runtime> Timeout<R> {
    /// Create a new timeout that expires after `default_timeout`
    ///
    /// # Panics
    /// Panics if `default_timeout` is longer than ~584 years
    #[must_use]
    pub fn new(runtime: R, default_timeout: Duration) -> Self {
        let epoch = runtime.now();
        let default_timeout = u64::try_from(default_timeout.as_nanos()).unwrap();
        Self {
            runtime,
            epoch,
            timeout_from_epoch_ns: default_timeout.into(),
            default_timeout: default_timeout.into(),
        }
    }

    fn elapsed(&self) -> Duration {
        self.runtime.now().duration_since(&self.epoch)
    }

    /// Reset the timeout to the default time.
    ///
    /// This function is cheap to call.
    ///
    /// # Panics
    /// Panics if over ~584 years have elapsed since the timer started.
    pub fn reset(&self) {
        self.timeout_from_epoch_ns.store(u64::try_from(self.elapsed().as_nanos()).unwrap() + self.default_timeout.load(Ordering::Acquire), Ordering::Release);
    }

    /// The default timeout. Timeout will be reset to this value upon a successful operation.
    pub fn default_timeout(&self) -> Duration {
        Duration::from_nanos(self.default_timeout.load(Ordering::Acquire))
    }
    /// Change the default timeout.
    ///
    /// Warning: if this timeout is shorter than previous one, it will only update after the
    /// previous timeout has expired!
    /// 
    /// # Panics
    /// Panics if `default_timeout` is longer than ~584 years
    pub fn set_default_timeout(&self, default_timeout: Duration) {
        self.default_timeout.store(u64::try_from(default_timeout.as_nanos()).unwrap(), Ordering::Release);
    }

    fn timeout_duration(&self) -> Option<Duration> {
        let elapsed_nanos = u64::try_from(self.elapsed().as_nanos()).unwrap();
        let target_nanos = self.timeout_from_epoch_ns.load(Ordering::Acquire);
        (elapsed_nanos < target_nanos).then(|| Duration::from_nanos(target_nanos - elapsed_nanos))
    }

    /// Wait for the timeout to expire
    ///
    /// This is a function that's expensive to start, so for best performance, only call it once
    /// per timer - launch it separately and call [`reset`](Timeout::reset) from the
    /// other futures (see the example in top-level documentation).
    pub async fn wait(&self) {
        pin_project_lite::pin_project! {
            struct SleepFuture<F: Sleep> {
                #[pin]
                inner: F,
            }
        }

        impl<F: Sleep> Future for SleepFuture<F> {
            type Output = ();
            fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
                self.project().inner.poll_sleep(cx)
            }
        }
        if let Some(timeout) = self.timeout_duration() {
            let future = self.runtime.create_sleep(timeout);
            let mut future = SleepFuture { inner: future };
            // SAFETY: the original future binding is shadowed,
            // so the unpinned binding can never be accessed again.
            // This is exactly the same code as the tokio::pin! macro
            let future = &mut unsafe { Pin::new_unchecked(&mut future) };
            while let Some(instant) = self.timeout_duration() {
                future.as_mut().project().inner.reset(instant);
                future.as_mut().await;
            }
        }
    }
}

#[cfg(feature = "wrapper")]
mod wrapper;
#[cfg(feature = "wrapper")]
pub use wrapper::Wrapper;

#[cfg(test)]
mod tests {
    use tokio::time::Instant;

    use crate::*;
    #[test]
    fn test_expiry() {
        let start = Instant::now();
        tokio_test::block_on(async {
            let timer = Timeout::new(runtime::Tokio::new(), Duration::from_secs(1));
            timer.wait().await;
        });
        assert!(start.elapsed() >= Duration::from_secs(1));
    }
    #[test]
    fn test_non_expiry() {
        let start = Instant::now();
        assert!(tokio_test::block_on(async {
            let timer = Timeout::new(runtime::Tokio::new(), Duration::from_secs(2));
            tokio::select! {
                _ = timer.wait() => {
                    false
                }
                _ = async {
                    tokio::time::sleep(Duration::from_secs(1)).await;
                    timer.reset();
                    tokio::time::sleep(Duration::from_secs(1)).await;
                } => {
                    true
                }
            }
        }));
        assert!(start.elapsed() >= Duration::from_secs(2));
    }
}
