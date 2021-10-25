// (c) Copyright 2021 Christian Saide
// SPDX-License-Identifier: MIT

use std::ops::{Index, IndexMut, RangeBounds};
use std::vec::Drain;
use std::{io, slice::Iter};

use super::{Closer, Error, Fd, IfReq, Opener, Result};

/// A named virtual device comprised of one or more virutal queues.
pub struct Device<T>(pub(super) Vec<T>);

impl<T> Device<T>
where
    T: Opener + Closer,
{
    /// Create a new set of queues for a device.
    pub fn new(name: &str, num_queues: usize) -> Result<(Self, String)> {
        if num_queues < 1 {
            return Err(Error::InvalidNumQueues);
        }

        let req = IfReq::new(name)?;
        let mut queues = Vec::with_capacity(num_queues);
        for _ in 0..num_queues {
            let queue = T::open(&req)?;
            queues.push(queue);
        }
        Ok((Self(queues), req.name()))
    }

    pub(super) fn get(&self, queue: usize) -> Result<&T> {
        if queue >= self.0.len() {
            Err(Error::from(queue))
        } else {
            Ok(&self.0[queue])
        }
    }

    pub fn close(&mut self) -> Result<()> {
        for mut queue in self.drain(..) {
            queue.close()?;
        }
        Ok(())
    }

    #[inline]
    pub fn drain<R>(&mut self, range: R) -> Drain<T>
    where
        R: RangeBounds<usize>,
    {
        self.0.drain(range)
    }

    #[inline]
    pub fn iter(&self) -> Iter<T> {
        self.0.iter()
    }
}

impl<T> Index<usize> for Device<T> {
    type Output = T;
    fn index(&self, index: usize) -> &T {
        &self.0[index]
    }
}

impl<T> IndexMut<usize> for Device<T> {
    fn index_mut(&mut self, index: usize) -> &mut T {
        &mut self.0[index]
    }
}

/// A synchronous virtual TUN device.
pub type Tun = Device<Fd>;

impl Tun {
    pub fn send_via(&self, queue: usize, datagram: &[u8]) -> io::Result<usize> {
        self.get(queue).map_err(|err| err.into_io())?.send(datagram)
    }

    pub fn recv_via(&self, queue: usize, datagram: &mut [u8]) -> io::Result<usize> {
        self.get(queue).map_err(|err| err.into_io())?.recv(datagram)
    }
}
