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
    /// Create a new set of queues for a device.
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

    /// Retrieve am immutable reference to the specified queue(s) if the suplied [SliceIndex] is inbounds.
    #[inline]
    pub fn get<I>(&self, index: I) -> Option<&I::Output>
    where
        I: SliceIndex<[Queue]>,
    {
        self.queues.get(index)
    }

    /// Retrieve a mutable reference to the specified queue(s) if the suplied [SliceIndex] is inbounds.
    #[inline]
    pub fn get_mut<I>(&mut self, index: I) -> Option<&mut I::Output>
    where
        I: SliceIndex<[Queue]>,
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

    /// Iterate over immutable instances internal queues.
    #[inline]
    pub fn iter(&self) -> Iter<Queue> {
        self.queues.iter()
    }

    /// Iterate over mutable instances of the internal queues.
    #[inline]
    pub fn iter_mut(&mut self) -> IterMut<Queue> {
        self.queues.iter_mut()
    }

    /// Send a packet via the specified TUN queue.
    pub fn send_via(&self, queue: usize, datagram: &[u8]) -> io::Result<usize> {
        self.get(queue)
            .ok_or_else(|| Error::from(queue).into_io())?
            .send(datagram)
    }

    /// Read a packet off the specified TUN queue.
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
        &self.queues[index]
    }
}

impl IndexMut<usize> for Tun {
    fn index_mut(&mut self, index: usize) -> &mut Queue {
        &mut self.queues[index]
    }
}
