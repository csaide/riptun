// (c) Copyright 2021 Christian Saide
// SPDX-License-Identifier: MIT

use super::{Closer, IfReq, Opener, Queue, Result};

use std::io;
use std::pin::Pin;
use std::task::{Context, Poll};

use async_io::Async;
use futures_io::{AsyncRead, AsyncWrite};

/// An async wrapper around the [Queue] object leveraging the [async_io] ecosystem.
pub struct AsyncStdQueue(Async<Queue>);

impl AsyncStdQueue {
    #[inline]
    pub(super) async fn readable(&self) -> io::Result<()> {
        self.0.readable().await
    }

    #[inline]
    pub(super) async fn writable(&self) -> io::Result<()> {
        self.0.writable().await
    }

    #[inline]
    pub(super) fn get_ref(&self) -> &Queue {
        self.0.get_ref()
    }

    /// Asynchrounously read a datagram off the underlying queue.
    pub async fn recv(&self, datagram: &mut [u8]) -> io::Result<usize> {
        self.0.read_with(|queue| queue.recv(datagram)).await
    }

    /// Asynchrounously write a datagram to the underlying queue.
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
    fn open(req: &IfReq) -> Result<Self> {
        let queue = Queue::open(req)?;
        let async_fd = Async::new(queue)?;
        Ok(Self(async_fd))
    }
}

impl Closer for AsyncStdQueue {
    fn close(&mut self) -> Result<()> {
        self.0.get_mut().close()
    }
}
