//! Traits needed for runtime-agnostic time measurement and sleeping
use core::{pin::Pin, time::Duration, task::{Poll, Context}};

#[cfg(feature = "tokio")]
mod tokio;
#[cfg(feature = "tokio")]
pub use self::tokio::Runtime as Tokio;

#[cfg(feature = "async-io")]
mod async_io;
#[cfg(feature = "async-io")]
pub use self::async_io::Runtime as AsyncIo;

/// A sleep future
pub trait Sleep {
    /// Set the future to time out in `timeout` from now
    fn reset(self: Pin<&mut Self>, timeout: Duration);
    /// Wait for the timeout to expire
    fn poll_sleep(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<()>;
}

/// Async runtime
pub trait Runtime {
    /// Sleep future type associated with this runtime
    type Sleep: Sleep;
    /// Instant type associated with this runtime
    type Instant: Instant;
    /// Create a new sleep future with given timeout
    fn create_sleep(&self, timeout: Duration) -> Self::Sleep;
    /// Get current time
    fn now(&self) -> Self::Instant;
}

/// An instant representing a point in time.
pub trait Instant {
    /// Duration since an earlier instant.
    fn duration_since(&self, earlier: &Self) -> Duration;
}
