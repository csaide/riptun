// (c) Copyright 2021 Christian Saide
// SPDX-License-Identifier: MIT

use super::*;

use std::io;
use std::pin::Pin;
use std::task::{Context, Poll};

use futures_util::ready;
use tokio::io::unix::AsyncFd;
use tokio::io::unix::AsyncFdReadyGuard;
use tokio::io::ReadBuf;
use tokio::io::{AsyncRead, AsyncWrite};

/// An async wrapper around the [Queue] object leveraging the [AsyncFd] struct internally
/// for async functionality within the `tokio` ecosystem.
///
/// This also implements both the [AsyncRead] and [AsyncWrite] enabling simple integration with the
/// greater ecosystem.
pub struct TokioQueue(AsyncFd<Queue>);

impl TokioQueue {
    /// Open a new async Queue based on the supplied [IfReq], exposing async capability for the
    /// tokio ecosystem.
    pub(crate) fn open(req: &IfReq) -> Result<Self> {
        let queue = Queue::open(req)?;
        queue.set_non_blocking(true)?;
        let async_fd = AsyncFd::new(queue)?;
        Ok(Self(async_fd))
    }

    /// Close the internal queue destroying this instance completely.
    pub fn close(&mut self) -> Result<()> {
        self.0.get_mut().close()
    }

    /// Wrapper around the internal [AsyncFd] structs [`AsyncFd::readable()`] call.
    #[inline]
    pub async fn readable(&self) -> io::Result<AsyncFdReadyGuard<'_, Queue>> {
        self.0.readable().await
    }

    /// Wrapper around the internal [AsyncFd] structs [`AsyncFd::writable()`] call.
    #[inline]
    pub async fn writable(&self) -> io::Result<AsyncFdReadyGuard<'_, Queue>> {
        self.0.writable().await
    }

    /// Return a reference to the internal [Queue]. This is generally used, when it's necessary
    /// to interact with the underlying [`Queue::recv()`] or [`Queue::send()`] methods.
    #[inline]
    pub fn get_ref(&self) -> &Queue {
        self.0.get_ref()
    }

    /// Asynchrounously read a datagram off the underlying queue. Looping over [`Queue::recv()`] calls
    /// using the [`AsyncFd::readable()`] + [`AsyncFdReadyGuard::try_io()`] calls waiting for either
    /// data to be ready and successfully read into the supplied buffer, or an error other than
    /// [`WouldBlock`][std::io::ErrorKind::WouldBlock] is encountered. Upon success the number of bytes
    /// read is returned, which will be between `0` and the length of the supplied buffer.
    ///
    /// # Errors
    /// On any error it should be assumed that no usable data was read into the buffer.
    pub async fn recv(&self, datagram: &mut [u8]) -> io::Result<usize> {
        loop {
            let mut guard = self.0.readable().await?;
            match guard.try_io(|queue| queue.get_ref().recv(datagram)) {
                Ok(res) => return res,
                Err(_) => continue,
            };
        }
    }

    /// Asynchrounously write a datagram to the underlying queue. Looping over [`Queue::send()`] calls
    /// using the [`AsyncFd::writable()`] + [`AsyncFdReadyGuard::try_io()`] calls waiting for either data
    /// to be ready and successfully sent from the supplied buffer, or an error other than
    /// [`WouldBlock`][std::io::ErrorKind::WouldBlock] is encountered. Upon success the number of bytes
    /// sent is returned, which will be between `0` and the length of the supplied buffer.
    ///
    /// # Errors
    /// On any error it should be assumed that the buffer was partially sent.
    pub async fn send(&self, datagram: &[u8]) -> io::Result<usize> {
        loop {
            let mut guard = self.0.writable().await?;
            match guard.try_io(|queue| queue.get_ref().send(datagram)) {
                Ok(res) => return res,
                Err(_) => continue,
            };
        }
    }
}

impl AsyncWrite for TokioQueue {
    fn poll_write(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        datagram: &[u8],
    ) -> Poll<io::Result<usize>> {
        loop {
            let mut guard = ready!(self.0.poll_write_ready(cx))?;
            match guard.try_io(|queue| queue.get_ref().send(datagram)) {
                Ok(res) => return Poll::Ready(res),
                Err(_) => continue,
            };
        }
    }

    fn poll_flush(self: Pin<&mut Self>, _: &mut Context<'_>) -> Poll<io::Result<()>> {
        // Flushing is a no-op on a char device.
        Poll::Ready(Ok(()))
    }

    fn poll_shutdown(mut self: Pin<&mut Self>, _: &mut Context<'_>) -> Poll<io::Result<()>> {
        self.0.get_mut().close().map_err(|err| err.into_io())?;
        Poll::Ready(Ok(()))
    }
}

impl AsyncRead for TokioQueue {
    fn poll_read(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        datagram: &mut ReadBuf<'_>,
    ) -> Poll<io::Result<()>> {
        loop {
            let mut guard = ready!(self.0.poll_read_ready(cx))?;
            let unfilled = unsafe { datagram.unfilled_mut() };
            match guard.try_io(|queue| queue.get_ref().recv_uninit(unfilled)) {
                Ok(res) => match res {
                    Ok(read) => {
                        unsafe { datagram.assume_init(read) };
                        datagram.advance(read);
                        return Poll::Ready(Ok(()));
                    }
                    Err(err) => return Poll::Ready(Err(err)),
                },
                Err(_) => continue,
            }
        }
    }
}

impl Opener for TokioQueue {
    #[inline]
    fn open(req: &IfReq) -> Result<Self> {
        Self::open(req)
    }
}
