use crate::runtime::Runtime;
use core::{
    pin::Pin,
    task::{Context, Poll},
};
use futures_io::{AsyncBufRead, AsyncRead, AsyncSeek, AsyncWrite};

use super::Wrapper;

#[cfg_attr(docsrs, doc(cfg(all(feature = "futures-io", feature = "read-write"))))]
impl<R: Runtime, T: AsyncRead> AsyncRead for Wrapper<'_, R, T> {
    fn poll_read(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &mut [u8],
    ) -> Poll<futures_io::Result<usize>> {
        let pinned = self.project();
        match pinned.inner.poll_read(cx, buf) {
            Poll::Ready(Ok(read)) if read > 0 => {
                pinned.timeout.as_ref().reset();
                Poll::Ready(Ok(read))
            }
            x => x,
        }
    }
}

#[cfg_attr(docsrs, doc(cfg(all(feature = "futures-io", feature = "read-write"))))]
impl<R: Runtime, T: AsyncWrite> AsyncWrite for Wrapper<'_, R, T> {
    fn poll_flush(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<futures_io::Result<()>> {
        self.project().inner.poll_flush(cx)
    }
    fn poll_close(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<futures_io::Result<()>> {
        self.project().inner.poll_close(cx)
    }
    fn poll_write(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &[u8],
    ) -> Poll<futures_io::Result<usize>> {
        let pinned = self.project();
        match pinned.inner.poll_write(cx, buf) {
            Poll::Ready(Ok(written)) if written > 0 => {
                pinned.timeout.as_ref().reset();
                Poll::Ready(Ok(written))
            }
            x => x,
        }
    }
    fn poll_write_vectored(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        bufs: &[futures_io::IoSlice<'_>],
    ) -> Poll<futures_io::Result<usize>> {
        let pinned = self.project();
        match pinned.inner.poll_write_vectored(cx, bufs) {
            Poll::Ready(Ok(written)) if written > 0 => {
                pinned.timeout.as_ref().reset();
                Poll::Ready(Ok(written))
            }
            x => x,
        }
    }
}

#[cfg_attr(docsrs, doc(cfg(all(feature = "futures-io", feature = "read-write"))))]
impl<R: Runtime, T: AsyncBufRead> AsyncBufRead for Wrapper<'_, R, T> {
    fn consume(self: Pin<&mut Self>, amt: usize) {
        self.project().inner.consume(amt);
    }
    fn poll_fill_buf(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
    ) -> Poll<futures_io::Result<&[u8]>> {
        let pinned = self.project();
        match pinned.inner.poll_fill_buf(cx) {
            Poll::Ready(Ok(bytes)) => {
                if !bytes.is_empty() {
                    pinned.timeout.as_ref().reset();
                }
                Poll::Ready(Ok(bytes))
            }
            Poll::Ready(Err(err)) => Poll::Ready(Err(err)),
            Poll::Pending => Poll::Pending,
        }
    }
}

#[cfg_attr(docsrs, doc(cfg(all(feature = "futures-io", feature = "read-write"))))]
impl<R: Runtime, T: AsyncSeek> AsyncSeek for Wrapper<'_, R, T> {
    fn poll_seek(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        pos: futures_io::SeekFrom,
    ) -> Poll<futures_io::Result<u64>> {
        let pinned = self.project();
        match pinned.inner.poll_seek(cx, pos) {
            Poll::Ready(Ok(pos)) => {
                pinned.timeout.as_ref().reset();
                Poll::Ready(Ok(pos))
            }
            x => x,
        }
    }
}
