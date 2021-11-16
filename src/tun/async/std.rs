// (c) Copyright 2021 Christian Saide
// SPDX-License-Identifier: MIT

use super::*;

use std::io;
use std::ops::{Index, IndexMut, RangeBounds};
use std::slice::{Iter, IterMut, SliceIndex};
use std::vec::{Drain, IntoIter};

use futures_util::future::select_all;

/// An asynchronous virtual TUN device based on the `async-std`/`smol` ecosystems.
pub struct AsyncStdTun {
    queues: Vec<AsyncStdQueue>,
    name: String,
}

impl AsyncStdTun {
    /// Create a new multi-queue async Tun device, supporting the `async-std`/`smol` ecosystems,
    /// using the specified name and number of queues. The name parameter can be augmented with `%d`
    /// to denote a OS determined incrementing ID to assign this device. To get the real device
    /// name call [`TokioTun::name()`].
    pub fn new(name: &str, num_queues: usize) -> Result<Self> {
        if num_queues < 1 {
            return Err(Error::InvalidNumQueues);
        }

        let (queues, name) = new_queues(name, num_queues)?;
        Ok(Self { queues, name })
    }

    /// Return the OS determined name of this device.
    #[inline]
    pub fn name(&self) -> &str {
        self.name.as_str()
    }

    /// Retrieve an immutable reference to the specified [AsyncStdQueue] if the suplied [SliceIndex]
    /// is inbounds.
    #[inline]
    pub fn get<I>(&self, index: I) -> Option<&AsyncStdQueue>
    where
        I: SliceIndex<[AsyncStdQueue], Output = AsyncStdQueue>,
    {
        self.queues.get(index)
    }

    /// Retrieve a mutable reference to the specified [AsyncStdQueue] if the suplied [SliceIndex] is
    /// inbounds.
    #[inline]
    pub fn get_mut<I>(&mut self, index: I) -> Option<&mut AsyncStdQueue>
    where
        I: SliceIndex<[AsyncStdQueue], Output = AsyncStdQueue>,
    {
        self.queues.get_mut(index)
    }

    /// Close the device destroying all internal queues.
    /// NOTE: If `drain` is called its on the caller to cleanup the queues.
    pub fn close(&mut self) -> Result<()> {
        for mut queue in self.drain(..) {
            queue.close()?;
        }
        Ok(())
    }

    /// Drain the internal queues, passing ownership of the queue and its lifecycle
    /// to the caller. This is useful in certain scenarios where extreme control over
    /// threading and I/O operations is desired.
    #[inline]
    pub fn drain<R>(&mut self, range: R) -> Drain<AsyncStdQueue>
    where
        R: RangeBounds<usize>,
    {
        self.queues.drain(range)
    }

    /// Iterate over immutable instances internal [AsyncStdQueue] instances.
    #[inline]
    pub fn iter(&self) -> Iter<AsyncStdQueue> {
        self.queues.iter()
    }

    /// Iterate over mutable instances of the internal [AsyncStdQueue] instances.
    #[inline]
    pub fn iter_mut(&mut self) -> IterMut<AsyncStdQueue> {
        self.queues.iter_mut()
    }

    /// Send a packet asynchronously to an available queue. This method handles collecting
    /// all of the [`AsyncStdQueue::writable()`] futures. Then leverages [`select_all`][futures_util::future::select_all]
    /// to await the first available queue to send the datagram via.
    pub async fn send(&self, datagram: &[u8]) -> io::Result<usize> {
        loop {
            // First collect all queue writable futures, pinning them as needed.
            let futures = self.iter().map(|queue| Box::pin(queue.writable()));

            // Select the first available queue to write to.
            let (result, idx, _) = select_all(futures).await;

            // Check to see if we errored, if so short circuit.
            if let Err(e) = result {
                return Err(e);
            }

            // Using the index returned from the above `select_all` call, retrieve
            // the queue in question, and attempt to send the datagram. Ensuring that
            // if the write fails due to EWOULDBLOCK/EAGAIN that the process is retried.
            let queue = self
                .get(idx)
                .ok_or_else(|| Error::InvalidQueue(idx).into_io())?;

            match queue.get_ref().send(datagram) {
                Err(e) if e.kind() == io::ErrorKind::WouldBlock => continue,
                result => return result,
            }
        }
    }

    /// Send a packet asynchronously via the specified TUN queue, see the [`AsyncStdQueue::send()`]
    /// documentation for more details.
    ///
    /// # Errors
    /// General I/O errors are possible, along with a [Error::InvalidQueue] if the specified
    /// queue is out of range for this device.
    pub async fn send_via(&self, queue: usize, datagram: &[u8]) -> io::Result<usize> {
        // Since we have a specific queue lets just retrieve the specified queue, erroring
        // if the specified queue is out of range.
        //
        // Then leverage the exposed async send on the queue to handle the heavy lifting.
        self.get(queue)
            .ok_or_else(|| Error::InvalidQueue(queue).into_io())?
            .send(datagram)
            .await
    }

    /// Receive a packet asynchronously from an available queue. This method handles collecting
    /// all of the [`AsyncStdQueue::readable()`] futures. Then leverages [`select_all`] to await the
    /// first available queue to send the datagram via.
    pub async fn recv(&self, datagram: &mut [u8]) -> io::Result<usize> {
        loop {
            // First collect all queue readable futures, pinning them as needed.
            let futures = self.iter().map(|queue| Box::pin(queue.readable()));

            // Select the first available queue with data to read.
            let (result, idx, _) = select_all(futures).await;

            // Check to see if we errored, if so short circuit.
            if let Err(e) = result {
                return Err(e);
            }

            // Using the index returned from the above `select_all` call, retrieve
            // the queue in question, and attempt to read the datagram. Ensuring that
            // if the read fails due to EWOULDBLOCK/EAGAIN that the process is retried.
            let queue = self
                .get(idx)
                .ok_or_else(|| Error::InvalidQueue(idx).into_io())?;
            match queue.get_ref().recv(datagram) {
                Err(e) if e.kind() == io::ErrorKind::WouldBlock => continue,
                result => return result,
            }
        }
    }

    /// Receive a packet asynchronously from the specified TUN queue, see the [`AsyncStdQueue::recv()`]
    /// documentation for more details.
    ///
    /// # Errors
    /// General I/O errors are possible, along with a [Error::InvalidQueue] if the specified
    /// queue is out of range for this device.
    pub async fn recv_via(&self, queue: usize, datagram: &mut [u8]) -> io::Result<usize> {
        // Since we have a specific queue lets just retrieve the specified queue, erroring
        // if the specified queue is out of range.
        //
        // Then leverage the exposed async recv on the queue to handle the heavy lifting.
        self.get(queue)
            .ok_or_else(|| Error::InvalidQueue(queue).into_io())?
            .recv(datagram)
            .await
    }
}

impl IntoIterator for AsyncStdTun {
    type Item = AsyncStdQueue;
    type IntoIter = IntoIter<AsyncStdQueue>;

    fn into_iter(self) -> Self::IntoIter {
        self.queues.into_iter()
    }
}

impl Index<usize> for AsyncStdTun {
    type Output = AsyncStdQueue;
    fn index(&self, index: usize) -> &AsyncStdQueue {
        self.queues.index(index)
    }
}

impl IndexMut<usize> for AsyncStdTun {
    fn index_mut(&mut self, index: usize) -> &mut AsyncStdQueue {
        self.queues.index_mut(index)
    }
}
