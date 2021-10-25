// (c) Copyright 2021 Christian Saide
// SPDX-License-Identifier: MIT

use super::*;

use std::io;

use futures_util::future::select_all;

impl DeviceQueue for AsyncStdFd {
    #[inline]
    fn open(req: &IfReq) -> Result<Self> {
        Self::open(req)
    }

    #[inline]
    fn close(&mut self) -> Result<()> {
        self.close().map_err(|err| err.into())
    }
}

/// An asynchronous virtual TUN device based on the async-std/async-io ecosystem.
pub type AsyncStdDev = Device<AsyncStdFd>;

impl AsyncStdDev {
    /// Send a packet asynchronously to an available queue.
    pub async fn send(&self, datagram: &[u8]) -> io::Result<usize> {
        loop {
            // First collect all queue writable futures, pinning them as needed.
            let futures = self.0.iter().map(|queue| Box::pin(queue.writable()));

            // Select the first available queue to write to.
            let (result, idx, _) = select_all(futures).await;

            // Check to see if we errored, if so short circuit.
            if let Err(e) = result {
                return Err(e);
            }

            // Using the index returned from the above `select_all` call, retrieve
            // the queue in question, and attempt to send the datagram. Ensuring that
            // if the write fails due to EWOULDBLOCK/EAGAIN that the process is retried.
            let queue = self.get(idx).map_err(|err| err.into_io())?.get_ref();
            match queue.send(datagram) {
                Err(e) if e.kind() == io::ErrorKind::WouldBlock => continue,
                result => return result,
            }
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
            let (result, idx, _) = select_all(futures).await;

            // Check to see if we errored, if so short circuit.
            if let Err(e) = result {
                return Err(e);
            }

            // Using the index returned from the above `select_all` call, retrieve
            // the queue in question, and attempt to read the datagram. Ensuring that
            // if the read fails due to EWOULDBLOCK/EAGAIN that the process is retried.
            let queue = self.get(idx).map_err(|err| err.into_io())?.get_ref();
            match queue.recv(datagram) {
                Err(e) if e.kind() == io::ErrorKind::WouldBlock => continue,
                result => return result,
            }
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
