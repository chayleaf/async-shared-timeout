use core::{future::Future, pin::Pin, task::{Poll, Context}};
#[cfg(feature = "std")]
use std::sync::Arc;
#[cfg(all(feature = "tokio", feature = "read-write"))]
mod tokio_read_write;
#[cfg(all(feature = "futures-io", feature = "read-write"))]
mod futures_io_read_write;
#[cfg(feature = "stream")]
mod stream;
#[cfg(all(feature = "std", unix))]
use std::os::unix::io::{AsRawFd, RawFd};

use crate::{Timeout, runtime::Runtime};

#[derive(Clone)]
enum CowTimeout<'a, R: Runtime> {
    #[cfg(feature = "std")]
    Arc(Arc<Timeout<R>>),
    Ref(&'a Timeout<R>),
}
impl<'a, R: Runtime> AsRef<Timeout<R>> for CowTimeout<'a, R> {
    fn as_ref(&self) -> &Timeout<R> {
        match self {
            #[cfg(feature = "std")]
            Self::Arc(x) => x,
            Self::Ref(x) => x,
        }
    }
}

pin_project_lite::pin_project! {
    /// A wrapper that wraps a future, a stream or an async reader/writer and resets the timeout
    /// upon a new event.
    ///
    /// **WARNING: THIS WILL NOT TIME OUT AUTOMATICALLY. THE TIMEOUT MUST BE AWAITED SOMEWHERE ELSE.**
    /// See example below.
    ///
    /// - In case of a [future](core::future::Future), timeout will be reset upon future completion
    /// - In case of an [`AsyncRead`](tokio::io::AsyncRead) object, timeout will be reset upon a
    ///   successful read or seek.
    /// - In case of an [`AsyncWrite`](tokio::io::AsyncWrite) object, timeout will be reset upon a
    ///   successful write. It will not be reset upon a shutdown or a flush, please notify me if you
    ///   think this should be changed!
    /// - In case of a [`Stream`](futures_core::Stream) object, timeout will be reset upon stream
    ///   advancement.
    /// 
    /// Since [`Wrapper::new`] accepts a shared reference to `Timeout`, you can make multiple
    /// objects use a single timeout. This means the timeout will only expire when *all* objects
    /// stopped having new events.
    ///
    /// # Example
    /// ```
    /// # async fn wrapper() -> std::io::Result<()> {
    /// # let remote_stream = tokio_test::io::Builder::new().build();
    /// # let local_stream = tokio_test::io::Builder::new().build();
    /// // Proxy with timeout
    /// use std::{io, time::Duration};
    /// use async_shared_timeout::{runtime, Timeout, Wrapper};
    ///
    /// let runtime = runtime::Tokio::new();
    /// let timeout_dur = Duration::from_secs(10);
    /// let timeout = Timeout::new(runtime, timeout_dur);
    /// let mut remote_stream = Wrapper::new(remote_stream, &timeout);
    /// let mut local_stream = Wrapper::new(local_stream, &timeout);
    /// let (copied_a_to_b, copied_b_to_a) = tokio::select! {
    ///     _ = timeout.wait() => {
    ///         return Err(io::Error::new(io::ErrorKind::TimedOut, "stream timeout"))
    ///     }
    ///     x = tokio::io::copy_bidirectional(&mut remote_stream, &mut local_stream) => {
    ///         x?
    ///     }
    /// };
    /// # drop((copied_a_to_b, copied_b_to_a));
    /// # Ok(())
    /// # }
    /// ```
    #[cfg_attr(docsrs, doc(cfg(feature = "wrapper")))]
    pub struct Wrapper<'a, R: Runtime, T> {
        #[pin]
        inner: T,
        timeout: CowTimeout<'a, R>,
    }
}

impl<'a, R: Runtime, T> Wrapper<'a, R, T> {
    /// Create a wrapper around an object that will update the given timeout upon successful
    /// operations
    /// 
    /// # Arguments
    /// 
    /// - `inner` - the object to be wrapped
    /// - `timeout` - a reference to the timeout to be used for operations on `inner`
    /// - `default_timeout` - on a successful operation, `timeout` will be [reset](`Timeout::reset`) to this value
    #[must_use]
    pub fn new(inner: T, timeout: &'a Timeout<R>) -> Self {
        Self {
            inner,
            timeout: CowTimeout::Ref(timeout),
        }
    }
    /// The timeout reference
    pub fn timeout(&self) -> &Timeout<R> {
        self.timeout.as_ref()
    }
    /// A reference to the underlying object
    pub fn inner(&self) -> &T {
        &self.inner
    }
    /// A mutable reference to the underlying object
    pub fn inner_mut(&mut self) -> &mut T {
        &mut self.inner
    }
}

#[cfg(feature = "std")]
#[cfg_attr(docsrs, doc(cfg(feature = "std")))]
impl<R: Runtime, T> Wrapper<'static, R, T> {
    /// Create a wrapper using a timeout behind an `Arc` pointer rather than a shared reference.
    /// See [`Wrapper::new`] for more info.
    #[must_use]
    pub fn new_arc(inner: T, timeout: Arc<Timeout<R>>) -> Self {
        Self {
            inner,
            timeout: CowTimeout::Arc(timeout),
        }
    }
}

impl<T, R: Runtime> AsRef<T> for Wrapper<'_, R, T> {
    fn as_ref(&self) -> &T {
        &self.inner
    }
}

impl<T, R: Runtime> AsMut<T> for Wrapper<'_, R, T> {
    fn as_mut(&mut self) -> &mut T {
        &mut self.inner
    }
}

impl<R: Runtime, T: Future> Future for Wrapper<'_, R, T> {
    type Output = T::Output;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let pinned = self.project();
        match pinned.inner.poll(cx) {
            Poll::Ready(x) => {
                pinned.timeout.as_ref().reset();
                Poll::Ready(x)
            }
            Poll::Pending => Poll::Pending,
        }
    }
}

#[cfg(all(feature = "std", unix))]
#[cfg_attr(docsrs, doc(cfg(all(feature = "std", unix))))]
impl<R: Runtime, T: AsRawFd> AsRawFd for Wrapper<'_, R, T> {
    fn as_raw_fd(&self) -> RawFd {
        self.inner.as_raw_fd()
    }
}
