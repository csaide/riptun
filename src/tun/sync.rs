// (c) Copyright 2021 Christian Saide
// SPDX-License-Identifier: MIT

use super::*;

use std::io;
use std::ops::{Index, IndexMut, RangeBounds};
use std::slice::{Iter, IterMut, SliceIndex};
use std::vec::{Drain, IntoIter};

/// A named virtual device comprised of one or more virutal queues.
pub struct Tun {
    queues: Vec<Queue>,
    name: String,
}

impl Tun {
    /// Create a new multi-queue Tun device using the specified name and number of queues.
    /// The name parameter can be augmented with `%d` to denote a OS determined incrementing
    /// ID to assign this device. To get the real device name call [`Tun::name()`].
    pub fn new(name: &str, num_queues: usize) -> Result<Self> {
        if num_queues < 1 {
            return Err(Error::InvalidNumQueues);
        }

        let (queues, name) = new_queues(name, num_queues)?;
        Ok(Self { queues, name })
    }

    /// Return the OS determined name of this device. Note this can and usually does differ somewhat from
    /// the supplied name during creation.
    #[inline]
    pub fn name(&self) -> &str {
        self.name.as_str()
    }

    /// Retrieve am immutable reference to the specified [Queue] if the suplied [SliceIndex] is inbounds.
    #[inline]
    pub fn get<I>(&self, index: I) -> Option<&Queue>
    where
        I: SliceIndex<[Queue], Output = Queue>,
    {
        self.queues.get(index)
    }

    /// Retrieve a mutable reference to the specified queue(s) if the suplied [SliceIndex] is inbounds.
    #[inline]
    pub fn get_mut<I>(&mut self, index: I) -> Option<&mut Queue>
    where
        I: SliceIndex<[Queue], Output = Queue>,
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
    pub fn drain<R>(&mut self, range: R) -> Drain<Queue>
    where
        R: RangeBounds<usize>,
    {
        self.queues.drain(range)
    }

    /// Iterate over immutable references to the internal [Queue] structs.
    #[inline]
    pub fn iter(&self) -> Iter<Queue> {
        self.queues.iter()
    }

    /// Iterate over mutable references to the internal [Queue] structs.
    #[inline]
    pub fn iter_mut(&mut self) -> IterMut<Queue> {
        self.queues.iter_mut()
    }

    /// Send a packet via the specified TUN queue, see the [`Queue::send()`] documentation for
    /// more details.
    ///
    /// # Errors
    /// General I/O errors are possible, along with a [Error::InvalidQueue] if the specified
    /// queue is out of range for this device.
    pub fn send_via(&self, queue: usize, datagram: &[u8]) -> io::Result<usize> {
        self.get(queue)
            .ok_or_else(|| Error::from(queue).into_io())?
            .send(datagram)
    }

    /// Read a packet off the specified TUN queue, see the [`Queue::recv()`] documentation for
    /// more details.
    ///
    /// # Errors
    /// General I/O errors are possible, along with a [Error::InvalidQueue] if the specified
    /// queue is out of range for this device.
    pub fn recv_via(&self, queue: usize, datagram: &mut [u8]) -> io::Result<usize> {
        self.get(queue)
            .ok_or_else(|| Error::from(queue).into_io())?
            .recv(datagram)
    }
}

impl IntoIterator for Tun {
    type Item = Queue;
    type IntoIter = IntoIter<Queue>;

    fn into_iter(self) -> Self::IntoIter {
        self.queues.into_iter()
    }
}

impl Index<usize> for Tun {
    type Output = Queue;
    fn index(&self, index: usize) -> &Queue {
        self.queues.index(index)
    }
}

impl IndexMut<usize> for Tun {
    fn index_mut(&mut self, index: usize) -> &mut Queue {
        self.queues.index_mut(index)
    }
}
