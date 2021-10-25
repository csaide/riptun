// (c) Copyright 2021 Christian Saide
// SPDX-License-Identifier: MIT

use super::*;

use std::io;

use futures_util::future::select_all;

impl DeviceQueue for TokioFd {
    #[inline]
    fn open(req: &IfReq) -> Result<Self> {
        Self::open(req)
    }

    #[inline]
    fn close(&mut self) -> Result<()> {
        self.close().map_err(|err| err.into())
    }
}

/// An asynchronous virtual TUN device based on the tokio ecosystem.
pub type TokioDev = Device<TokioFd>;

impl TokioDev {
    /// Send a packet asynchronously to an available queue.
    pub async fn send(&self, datagram: &[u8]) -> io::Result<usize> {
        loop {
            // First collect all queue writable futures, pinning them as needed.
            let futures = self.0.iter().map(|queue| Box::pin(queue.writable()));

            // Select the first available queue to write to.
            let (result, _, _) = select_all(futures).await;

            // Unwrap the Result returning the AsyncReadyGuard or propagating the error upstream.
            let mut guard = match result {
                Ok(guard) => guard,
                Err(e) => return Err(e),
            };

            // Using the AsyncReadyGuard try to preform the requested I/O operation,
            // if the result is an error we know it would have blocked, so retry the whole
            // process again.
            match guard.try_io(|queue| queue.get_ref().send(datagram)) {
                Ok(res) => return res,
                Err(_) => continue,
            };
        }
    }

    /// Send a packet asynchronously to the specified queue.
    pub async fn send_via(&self, queue: usize, datagram: &[u8]) -> io::Result<usize> {
        // Since we have a specific queue lets just retrieve the specified queue, erroring
        // if the specified queue is out of range.
        //
        // Then leverage the exposed async send on the queue to handle the heavy lifting.
        self.get(queue)
            .map_err(|err| err.into_io())?
            .send(datagram)
            .await
    }

    /// Receive a packet asynchronously from an available queue.
    pub async fn recv(&self, datagram: &mut [u8]) -> io::Result<usize> {
        loop {
            // First collect all queue readable futures, pinning them as needed.
            let futures = self.0.iter().map(|queue| Box::pin(queue.readable()));

            // Select the first available queue with data to read.
            let (result, _, _) = select_all(futures).await;

            // Unwrap the Result returning the AsyncReadyGuard or propagating the error upstream.
            let mut guard = match result {
                Ok(guard) => guard,
                Err(e) => return Err(e),
            };

            // Using the AsyncReadyGuard try to preform the requested I/O operation,
            // if the result is an error we know it would have blocked, so retry the whole
            // process again.
            match guard.try_io(|queue| queue.get_ref().recv(datagram)) {
                Ok(res) => return res,
                Err(_) => continue,
            };
        }
    }

    /// Receive a packet asynchronously from the specified queue.
    pub async fn recv_via(&self, queue: usize, datagram: &mut [u8]) -> io::Result<usize> {
        // Since we have a specific queue lets just retrieve the specified queue, erroring
        // if the specified queue is out of range.
        //
        // Then leverage the exposed async recv on the queue to handle the heavy lifting.
        self.get(queue)
            .map_err(|err| err.into_io())?
            .recv(datagram)
            .await
    }
}
