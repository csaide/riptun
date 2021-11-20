// (c) Copyright 2021 Christian Saide
// SPDX-License-Identifier: MIT

use super::*;

use std::io;
use std::ops::{Index, IndexMut, RangeBounds};
use std::slice::{Iter, IterMut, SliceIndex};
use std::vec::{Drain, IntoIter};

use futures_util::future::select_all;

/// An asynchronous virtual TUN device based on the `tokio` ecosystem.
pub struct TokioTun {
    queues: Vec<TokioQueue>,
    name: String,
}

impl TokioTun {
    /// Create a new multi-queue async Tun device, supporting the `tokio` ecosystem, using the
    /// specified name and number of queues. The name parameter can be augmented with `%d` to
    /// denote a OS determined incrementing ID to assign this device. To get the real device
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

    /// Retrieve an immutable reference to the specified queue(s) if the suplied [SliceIndex] is inbounds.
    #[inline]
    pub fn get<I>(&self, index: I) -> Option<&I::Output>
    where
        I: SliceIndex<[TokioQueue]>,
    {
        self.queues.get(index)
    }

    /// Retrieve a mutable reference to the specified queue(s) if the suplied [SliceIndex] is inbounds.
    #[inline]
    pub fn get_mut<I>(&mut self, index: I) -> Option<&mut I::Output>
    where
        I: SliceIndex<[TokioQueue]>,
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
    pub fn drain<R>(&mut self, range: R) -> Drain<TokioQueue>
    where
        R: RangeBounds<usize>,
    {
        self.queues.drain(range)
    }

    /// Iterate over immutable instances internal queues.
    #[inline]
    pub fn iter(&self) -> Iter<TokioQueue> {
        self.queues.iter()
    }

    /// Iterate over mutable instances of the internal queues.
    #[inline]
    pub fn iter_mut(&mut self) -> IterMut<TokioQueue> {
        self.queues.iter_mut()
    }

    /// Send a packet asynchronously to an available queue. This method handles collecting
    /// all of the [`TokioQueue::writable()`] futures. Then leverages [`select_all()`] to await the
    /// first available queue to send the datagram via.
    pub async fn send(&self, datagram: &[u8]) -> io::Result<usize> {
        loop {
            // First collect all queue writable futures, pinning them as needed.
            let futures = self.iter().map(|queue| Box::pin(queue.writable()));

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

    /// Send a packet asynchronously via the specified TUN queue, see the [`TokioQueue::send()`]
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
    /// all of the [`TokioQueue::readable()`] futures. Then leverages [`select_all()`] to await the
    /// first available queue to send the datagram via.
    pub async fn recv(&self, datagram: &mut [u8]) -> io::Result<usize> {
        loop {
            // First collect all queue readable futures, pinning them as needed.
            let futures = self.iter().map(|queue| Box::pin(queue.readable()));

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

    /// Receive a packet asynchronously from the specified TUN queue, see the [`TokioQueue::recv()`]
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

impl IntoIterator for TokioTun {
    type Item = TokioQueue;
    type IntoIter = IntoIter<TokioQueue>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        self.queues.into_iter()
    }
}

impl Index<usize> for TokioTun {
    type Output = TokioQueue;

    #[inline]
    fn index(&self, index: usize) -> &TokioQueue {
        self.queues.index(index)
    }
}

impl IndexMut<usize> for TokioTun {
    #[inline]
    fn index_mut(&mut self, index: usize) -> &mut TokioQueue {
        self.queues.index_mut(index)
    }
}
