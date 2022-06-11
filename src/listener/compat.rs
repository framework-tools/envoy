use std::{pin::Pin, task::Context};

use futures_util::ready;
use tokio::io::ReadBuf;

pub struct TokioCompat<T>(T);

pub trait TokioCompatExt: async_std::io::Read + async_std::io::Write + Sized {
    #[inline]
    fn compat(self) -> TokioCompat<Self> {
        TokioCompat(self)
    }
}

impl<T: tokio::io::AsyncRead + Unpin> async_std::io::Read for TokioCompat<T> {
    #[inline]
    fn poll_read(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &mut [u8],
    ) -> std::task::Poll<Result<usize, async_std::io::Error>> {
        let current_size = buf.len();
        let result = Pin::new(&mut self.0).poll_read(cx, &mut ReadBuf::new(buf));
        let diff = buf.len() - current_size;
        match ready!(result) {
            Ok(()) => std::task::Poll::Ready(Ok(diff)),
            Err(e) => std::task::Poll::Ready(Err(e)),
        }
    }
}

impl<T: tokio::io::AsyncWrite + Unpin> async_std::io::Write for TokioCompat<T> {
    #[inline]
    fn poll_write(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &[u8],
    ) -> std::task::Poll<Result<usize, async_std::io::Error>> {
        Pin::new(&mut self.0).poll_write(cx, buf)
    }

    #[inline]
    fn poll_flush(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> std::task::Poll<Result<(), async_std::io::Error>> {
        Pin::new(&mut self.0).poll_flush(cx)
    }

    #[inline]
    fn poll_close(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> std::task::Poll<Result<(), async_std::io::Error>> {
        Pin::new(&mut self.0).poll_shutdown(cx)
    }
}