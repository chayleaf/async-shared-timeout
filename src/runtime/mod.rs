use core::{pin::Pin, time::Duration, task::{Poll, Context}};

#[cfg(feature = "tokio")]
#[cfg_attr(docsrs, doc(cfg(all(feature = "tokio"))))]
pub mod tokio;
#[cfg(feature = "tokio")]
#[cfg_attr(docsrs, doc(cfg(all(feature = "tokio"))))]
pub type Tokio = tokio::Runtime;

#[cfg(feature = "async-io")]
#[cfg_attr(docsrs, doc(cfg(all(feature = "async-io"))))]
pub mod async_io;
#[cfg(feature = "async-io")]
#[cfg_attr(docsrs, doc(cfg(all(feature = "async-io"))))]
pub type AsyncIo = async_io::Runtime;

/// A sleep future
pub trait Sleep {
    /// Set the future to time out in `timeout` from now
    fn reset(self: Pin<&mut Self>, timeout: Duration);
    /// Wait for the timeout to expire
    fn poll_sleep(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<()>;
}

/// Async runtime
pub trait Runtime {
    type Sleep: Sleep;
    type Instant: Instant;
    fn create_sleep(&self, timeout: Duration) -> Self::Sleep;
    fn now(&self) -> Self::Instant;
}

pub trait Instant {
    fn duration_since(&self, earlier: &Self) -> Duration;
}
