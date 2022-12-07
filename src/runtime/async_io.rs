use core::{future::Future, task::{Poll, Context}, time::Duration, pin::Pin};
use std::time::Instant;
use async_io::Timer;

#[derive(Copy, Clone, Default)]
pub struct Runtime {
    _private: (),
}

impl Runtime {
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

struct PendingFuture;
impl Future for PendingFuture {
    type Output = ();
    fn poll(self: Pin<&mut Self>, _: &mut Context<'_>) -> Poll<Self::Output> {
        Poll::Pending
    }
}

impl super::Sleep for Timer {
    fn poll_sleep(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<()> {
        match self.poll(cx) {
            Poll::Ready(_) => Poll::Ready(()),
            Poll::Pending => Poll::Pending,
        }
    }
    fn reset(mut self: Pin<&mut Self>, timeout: Duration) {
        self.set_after(timeout);
    }
}
