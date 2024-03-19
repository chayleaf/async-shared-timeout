use core::{
    future::Future,
    pin::Pin,
    task::{Context, Poll},
    time::Duration,
};
use tokio::time::{self, Instant, Sleep};

/// Tokio runtime implementation
#[derive(Copy, Clone, Default)]
#[cfg_attr(docsrs, doc(cfg(feature = "tokio")))]
pub struct Runtime {
    _private: (),
}

impl Runtime {
    /// Create a new Tokio runtime object. Current runtime will be used for sleeping. Launch
    /// [`Timeout::wait`](`crate::Timeout::wait`) in the correct context if necessary.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }
}

impl super::Runtime for Runtime {
    type Sleep = Sleep;
    type Instant = Instant;
    fn create_sleep(&self, timeout: Duration) -> Self::Sleep {
        time::sleep(timeout)
    }
    fn now(&self) -> Self::Instant {
        Instant::now()
    }
}

impl super::Instant for Instant {
    fn duration_since(&self, earlier: &Self) -> Duration {
        self.duration_since(*earlier)
    }
}

impl super::Sleep for Sleep {
    fn poll_sleep(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<()> {
        self.poll(cx)
    }
    fn reset(self: Pin<&mut Self>, timeout: Duration) {
        self.reset(time::Instant::now() + timeout);
    }
}
