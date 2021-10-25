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

/// An async wrapper around the [Fd] object.
pub struct TokioFd(AsyncFd<Fd>);

impl TokioFd {
    // Open a new non-blocking async Queue
    pub(super) fn open(req: &req::IfReq) -> Result<Self> {
        let queue = Fd::open(req)?;
        queue.set_non_blocking(true)?;
        let async_fd = AsyncFd::new(queue)?;
        Ok(Self(async_fd))
    }

    /// Close the underlying queue.
    pub fn close(&mut self) -> io::Result<()> {
        self.0.get_mut().close()
    }

    pub(super) async fn readable<'a>(&'a self) -> io::Result<AsyncFdReadyGuard<'a, Fd>> {
        self.0.readable().await
    }

    pub(super) async fn writable<'a>(&'a self) -> io::Result<AsyncFdReadyGuard<'a, Fd>> {
        self.0.writable().await
    }

    /// Asynchrounously read a datagram off the underlying queue.
    pub async fn recv(&self, datagram: &mut [u8]) -> io::Result<usize> {
        loop {
            let mut guard = self.0.readable().await?;
            match guard.try_io(|queue| queue.get_ref().recv(datagram)) {
                Ok(res) => return res,
                Err(_) => continue,
            };
        }
    }

    /// Asynchrounously write a datagrom to the underlying queue.
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

impl AsyncWrite for TokioFd {
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
        Poll::Ready(Ok(()))
    }

    fn poll_shutdown(mut self: Pin<&mut Self>, _: &mut Context<'_>) -> Poll<io::Result<()>> {
        self.0.get_mut().close()?;
        Poll::Ready(Ok(()))
    }
}

impl AsyncRead for TokioFd {
    fn poll_read(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        datagram: &mut ReadBuf<'_>,
    ) -> Poll<io::Result<()>> {
        loop {
            let mut guard = ready!(self.0.poll_read_ready(cx))?;
            match guard.try_io(|queue| queue.get_ref().recv(datagram.initialize_unfilled())) {
                Ok(res) => match res {
                    Ok(read) => {
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
