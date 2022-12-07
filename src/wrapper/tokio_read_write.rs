use core::{pin::Pin, task::{Context, Poll}};
use std::io;
use tokio::io::{AsyncRead, AsyncWrite, AsyncSeek, AsyncBufRead};
use crate::runtime::Runtime;

use super::Wrapper;

impl<R: Runtime, T: AsyncRead> AsyncRead for Wrapper<'_, R, T> {
    fn poll_read(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &mut tokio::io::ReadBuf<'_>,
    ) -> Poll<io::Result<()>> {
        let pinned = self.project();
        match pinned.inner.poll_read(cx, buf) {
            Poll::Ready(Ok(())) if !buf.filled().is_empty() => {
                pinned.timeout.as_ref().reset();
                Poll::Ready(Ok(()))
            }
            x => x,
        }
    }
}

impl<R: Runtime, T: AsyncWrite> AsyncWrite for Wrapper<'_, R, T> {
    fn is_write_vectored(&self) -> bool {
        self.inner.is_write_vectored()
    }
    fn poll_flush(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), io::Error>> {
        self.project().inner.poll_flush(cx)
    }
    fn poll_shutdown(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), io::Error>> {
        self.project().inner.poll_shutdown(cx)
    }
    fn poll_write(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &[u8],
    ) -> Poll<Result<usize, io::Error>> {
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
        bufs: &[io::IoSlice<'_>],
    ) -> Poll<Result<usize, io::Error>> {
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

impl<R: Runtime, T: AsyncBufRead> AsyncBufRead for Wrapper<'_, R, T> {
    fn consume(self: Pin<&mut Self>, amt: usize) {
        self.project().inner.consume(amt);
    }
    fn poll_fill_buf(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<io::Result<&[u8]>> {
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

impl<R: Runtime, T: AsyncSeek> AsyncSeek for Wrapper<'_, R, T> {
    fn start_seek(self: Pin<&mut Self>, position: io::SeekFrom) -> io::Result<()> {
        self.project().inner.start_seek(position)
    }
    fn poll_complete(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<io::Result<u64>> {
        let pinned = self.project();
        match pinned.inner.poll_complete(cx) {
            Poll::Ready(Ok(pos)) => {
                pinned.timeout.as_ref().reset();
                Poll::Ready(Ok(pos))
            }
            x => x,
        }
    }
}
