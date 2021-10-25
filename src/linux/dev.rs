// (c) Copyright 2021 Christian Saide
// SPDX-License-Identifier: MIT

use std::ops::RangeBounds;
use std::vec::Drain;
use std::{io, slice::Iter};

use super::{Error, Fd, IfReq, Result};

pub trait DeviceQueue: Sized {
    fn open(req: &IfReq) -> Result<Self>;
    fn close(&mut self) -> Result<()>;
}

/// A named virtual device comprised of one or more virutal queues.
pub struct Device<T>(pub(super) Vec<T>);

impl<T> Device<T>
where
    T: DeviceQueue,
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

impl DeviceQueue for Fd {
    #[inline]
    fn open(req: &IfReq) -> Result<Self> {
        Self::open(req)
    }

    #[inline]
    fn close(&mut self) -> Result<()> {
        self.close().map_err(|err| err.into())
    }
}

/// A synchronous virtual TUN device.
pub type Dev = Device<Fd>;

impl Dev {
    pub fn send(&self, queue: usize, datagram: &[u8]) -> io::Result<usize> {
        self.get(queue).map_err(|err| err.into_io())?.send(datagram)
    }

    pub fn recv(&self, queue: usize, datagram: &mut [u8]) -> io::Result<usize> {
        self.get(queue).map_err(|err| err.into_io())?.recv(datagram)
    }
}
