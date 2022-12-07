use core::{future::Future, task::{Poll, Context}, time::Duration, pin::Pin};
use std::time::Instant;
use async_io::Timer;

/// async-io runtime implementation
#[derive(Copy, Clone, Default)]
#[cfg_attr(docsrs, doc(cfg(feature = "async-io")))]
pub struct Runtime {
    _private: (),
}

impl Runtime {
    /// Create a new async-io runtime object
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }
}

impl super::Runtime for Runtime {
    type Sleep = Timer;
    type Instant = Instant;
    fn create_sleep(&self, timeout: Duration) -> Self::Sleep {
        Timer::after(timeout)
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

impl super::Sleep for Timer {
    fn poll_sleep(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<()> {
        self.poll(cx).map(|_| ())
    }
    fn reset(mut self: Pin<&mut Self>, timeout: Duration) {
        self.set_after(timeout);
    }
}
