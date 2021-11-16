// (c) Copyright 2021 Christian Saide
// SPDX-License-Identifier: MIT

use super::{Error, IfReq, Opener, Result};

use nix::{fcntl::OFlag, libc};

use std::io::{self, Read, Write};
use std::mem::MaybeUninit;
use std::os::unix::prelude::{AsRawFd, RawFd};

const PATH: &[u8] = b"/dev/net/tun\0";

nix::ioctl_write_int!(create_queue, b'T', 202);

#[cfg(target_pointer_width = "64")]
type PointerWidth = u64;
#[cfg(target_pointer_width = "32")]
type PointerWidth = u32;
#[cfg(target_pointer_width = "16")]
type PointerWidth = u16;

/// A raw TUN/TAP queue wrapping all I/O for both sync and async operations.
#[derive(Clone)]
pub struct Queue(RawFd);

impl Queue {
    /// Open a new queue using the supplied [IfReq], exposing a synchronous blocking queue.
    pub(crate) fn open(req: &IfReq) -> Result<Self> {
        let fd = unsafe { libc::open(PATH.as_ptr() as *const libc::c_char, libc::O_RDWR) };
        if fd < 0 {
            return Err(Error::FS {
                path: unsafe { String::from_utf8_unchecked(PATH.to_vec()) },
                source: io::Error::last_os_error(),
            });
        }

        let ret = unsafe { create_queue(fd, req as *const IfReq as PointerWidth)? };
        if ret >= 1 {
            return Err(Error::from(ret as i32));
        }
        Ok(Self(fd))
    }

    /// Close the internal queue destroying this instance completely.
    pub fn close(&mut self) -> Result<()> {
        let ret = unsafe { libc::close(self.0) };
        if ret < 0 {
            Err(Error::errno())
        } else {
            self.0 = -1;
            Ok(())
        }
    }

    /// Either enable or disable non-blocking mode on the underlying file descriptor.
    pub fn set_non_blocking(&self, on: bool) -> Result<()> {
        let flags =
            nix::fcntl::fcntl(self.0, nix::fcntl::FcntlArg::F_GETFL).map_err(Error::from)?;

        let mut flags = OFlag::from_bits(flags).unwrap_or(OFlag::O_RDWR);
        if on && !flags.contains(OFlag::O_NONBLOCK) {
            flags.insert(OFlag::O_NONBLOCK);
        } else if !on && flags.contains(OFlag::O_NONBLOCK) {
            flags.remove(OFlag::O_NONBLOCK);
        } else {
            return Ok(());
        }

        nix::fcntl::fcntl(self.0, nix::fcntl::FcntlArg::F_SETFL(flags))
            .map(|_| ())
            .map_err(Error::from)
    }

    /// Write the datagram to the underlying file descriptor injecting the data into the hosts networking
    /// stack. This call wraps the raw [`libc::write()`] call returning the number of bytes written from the
    /// buffer.
    ///
    /// In non-blocking mode this can and will return [`WouldBlock`][std::io::ErrorKind::WouldBlock] and that should
    /// be used as an indication that the queue is not ready for sending data, and be re-polled for readiness.
    ///
    /// # Errors
    /// On any error it should be assumed that the buffer was partially sent.
    pub fn send(&self, datagram: &[u8]) -> io::Result<usize> {
        let count = datagram.len();
        let written = unsafe {
            let ptr = datagram.as_ptr();
            libc::write(self.0, ptr as *const libc::c_void, count)
        };

        if written < 0 {
            Err(io::Error::last_os_error())
        } else {
            Ok(written as usize)
        }
    }

    /// Read data from the underlying file descriptor into the supplied datagram, reading data from the hosts networking
    /// stack. This call wraps the raw [`libc::read()`] call returning the number of bytes read into the supplied datagram.
    ///
    /// In non-blocking mode this can and will return [`WouldBlock`][std::io::ErrorKind::WouldBlock] and that should
    /// be used as an indication that the queue is not ready for reading data, and be re-polled for readiness.
    ///
    /// # Errors
    /// On any error it should be assumed that no usable data was read into the buffer.
    #[inline]
    pub fn recv(&self, datagram: &mut [u8]) -> io::Result<usize> {
        unsafe { self.recv_int(datagram.as_mut_ptr(), datagram.len()) }
    }

    /// Read data from the underlying file descriptor into the supplied datagram, reading data from the hosts networking
    /// stack, using uninitialized memory. This call is analogous to the [`Queue::recv()`] function but allows for using
    /// uninitialized memory buffers.
    ///
    /// # Safety
    /// The caller should never use data in the supplied datagram that is greater than the returned read count.
    ///
    /// # Errors
    /// On any error it should be assumed that no usable data was read into the buffer.
    #[inline]
    pub fn recv_uninit(&self, datagram: &mut [MaybeUninit<u8>]) -> io::Result<usize> {
        unsafe { self.recv_int(datagram.as_mut_ptr(), datagram.len()) }
    }

    unsafe fn recv_int<T>(&self, ptr: *mut T, count: usize) -> io::Result<usize> {
        let read = libc::read(self.0, ptr as *mut libc::c_void, count);
        if read < 0 {
            Err(io::Error::last_os_error())
        } else {
            Ok(read as usize)
        }
    }
}

impl AsRawFd for Queue {
    fn as_raw_fd(&self) -> RawFd {
        self.0
    }
}

impl Read for Queue {
    #[inline]
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        self.recv(buf)
    }
}

impl Write for Queue {
    #[inline]
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        self.send(buf)
    }

    #[inline]
    fn flush(&mut self) -> std::io::Result<()> {
        // TUN queues are character devices under the hood no flushing needed.
        Ok(())
    }
}

impl Opener for Queue {
    #[inline]
    fn open(req: &IfReq) -> Result<Self> {
        Self::open(req)
    }
}
