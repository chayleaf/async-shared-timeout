use core::{pin::Pin, task::{Context, Poll}};

use futures_core::Stream;
use crate::runtime::Runtime;

use super::Wrapper;

#[cfg_attr(docsrs, doc(cfg(feature = "stream")))]
impl<R: Runtime, T: Stream> Stream for Wrapper<'_, R, T> {
    type Item = T::Item;
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.inner.size_hint()
    }
    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let pinned = self.project();
        match pinned.inner.poll_next(cx) {
            Poll::Ready(x) => {
                pinned.timeout.as_ref().reset();
                Poll::Ready(x)
            }
            Poll::Pending => Poll::Pending
        }
    }
}

