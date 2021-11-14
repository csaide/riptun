// (c) Copyright 2021 Christian Saide
// SPDX-License-Identifier: MIT

use super::{IfReq, Opener, Queue, Result};

use std::io;
use std::pin::Pin;
use std::task::{Context, Poll};

use async_io::Async;
use futures_io::{AsyncRead, AsyncWrite};

/// An async wrapper around the [Queue] object leveraging the [Async] struct internally
/// for async functionality.
///
/// This also implements both the [AsyncRead] and [AsyncWrite] enabling simple integration
/// with both the `async-std` and `smol` ecosystems.
pub struct AsyncStdQueue(Async<Queue>);

impl AsyncStdQueue {
    /// Open a new async Queue based on the supplied [IfReq], exposing async capability for the
    /// async-std/smol ecosystems.
    pub(crate) fn open(req: &IfReq) -> Result<Self> {
        let queue = Queue::open(req)?;
        let async_fd = Async::new(queue)?;
        Ok(Self(async_fd))
    }

    /// Close the internal queue destroying this instance completely.
    pub fn close(&mut self) -> Result<()> {
        self.0.get_mut().close()
    }

    /// Wrapper around the [Async] struct's [`Async::readable()`] call.
    #[inline]
    pub async fn readable(&self) -> io::Result<()> {
        self.0.readable().await
    }

    /// Wrapper around the [Async] struct's [`Async::writable()`] call.
    #[inline]
    pub async fn writable(&self) -> io::Result<()> {
        self.0.writable().await
    }

    /// Return a reference to the internal [Queue]. This is generally used, when it's necessary
    /// to interact with the underlying [`Queue::recv()`] or [`Queue::send()`] methods.
    #[inline]
    pub fn get_ref(&self) -> &Queue {
        self.0.get_ref()
    }

    /// Asynchrounously read a datagram off the underlying queue. Looping over [`Queue::recv()`] calls
    /// using the [`Async::read_with()`] call waiting for either data to be ready and successfully read
    /// into the supplied buffer, or an error other than [`WouldBlock`][std::io::ErrorKind::WouldBlock]
    /// is encountered. Upon success the number of bytes read is returned, which will be between `0` and
    /// the length of the supplied buffer.
    ///
    /// # Errors
    /// On any error it should be assumed that no usable data was read into the buffer.
    pub async fn recv(&self, datagram: &mut [u8]) -> io::Result<usize> {
        self.0.read_with(|queue| queue.recv(datagram)).await
    }

    /// Asynchrounously write a datagram to the underlying queue. Looping over [`Queue::send()`] calls
    /// using the [`Async::write_with()`] call waiting for either data to be ready and successfully sent
    /// from the supplied buffer, or an error other than [`WouldBlock`][std::io::ErrorKind::WouldBlock]
    /// is encountered. Upon success the number of bytes sent is returned, which will be between `0` and
    /// the length of the supplied buffer.
    ///
    /// # Errors
    /// On any error it should be assumed that the buffer was partially sent.
    pub async fn send(&self, datagram: &[u8]) -> io::Result<usize> {
        self.0.write_with(|queue| queue.send(datagram)).await
    }
}

impl AsyncWrite for AsyncStdQueue {
    fn poll_write(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &[u8],
    ) -> Poll<io::Result<usize>> {
        let inner = Pin::new(&mut self.get_mut().0);
        inner.poll_write(cx, buf)
    }

    fn poll_flush(self: Pin<&mut Self>, _: &mut Context<'_>) -> Poll<io::Result<()>> {
        // Flushing is a no-op on a char device.
        Poll::Ready(Ok(()))
    }

    fn poll_close(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<io::Result<()>> {
        let inner = Pin::new(&mut self.get_mut().0);
        inner.poll_close(cx)
    }
}

impl AsyncRead for AsyncStdQueue {
    fn poll_read(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &mut [u8],
    ) -> std::task::Poll<io::Result<usize>> {
        let inner = Pin::new(&mut self.get_mut().0);
        inner.poll_read(cx, buf)
    }
}

impl Opener for AsyncStdQueue {
    #[inline]
    fn open(req: &IfReq) -> Result<Self> {
        Self::open(req)
    }
}
