// (c) Copyright 2021 Christian Saide
// SPDX-License-Identifier: MIT

use nix::{fcntl::OFlag, libc};

use std::{
    io::{self, Read, Write},
    mem::MaybeUninit,
    os::unix::prelude::{AsRawFd, RawFd},
};

use super::{req, Closer, Error, Opener, Result};

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
pub struct Fd(RawFd);

impl AsRawFd for Fd {
    fn as_raw_fd(&self) -> RawFd {
        self.0
    }
}

impl Fd {
    /// Either enable or disable non-blocking mode on the underlying file descriptor.
    pub fn set_non_blocking(&self, on: bool) -> Result<()> {
        let flags =
            nix::fcntl::fcntl(self.0, nix::fcntl::FcntlArg::F_GETFL).map_err(Error::from)?;

        let mut flags = OFlag::from_bits(flags).unwrap_or(OFlag::O_RDWR);
        if on {
            flags.insert(OFlag::O_NONBLOCK);
        } else {
            flags.remove(OFlag::O_NONBLOCK);
        }

        nix::fcntl::fcntl(self.0, nix::fcntl::FcntlArg::F_SETFL(flags))
            .map(|_| ())
            .map_err(Error::from)
    }

    /// Write the datagram to the underlying file descriptor injecting the data into the hosts networking
    /// stack.
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
    /// stack.
    pub fn recv(&self, datagram: &mut [u8]) -> io::Result<usize> {
        let count = datagram.len();
        let read = unsafe {
            let ptr = datagram.as_mut_ptr();
            libc::read(self.0, ptr as *mut libc::c_void, count)
        };

        if read < 0 {
            Err(io::Error::last_os_error())
        } else {
            Ok(read as usize)
        }
    }
    /// Read data from the underlying file descriptor into the supplied datagram, reading data from the hosts networking
    /// stack, using uninitialized memory.
    pub fn recv_uninit(&self, datagram: &mut [MaybeUninit<u8>]) -> io::Result<usize> {
        let count = datagram.len();
        let read = unsafe {
            let ptr = datagram.as_mut_ptr();
            libc::read(self.0, ptr as *mut libc::c_void, count)
        };

        if read < 0 {
            Err(io::Error::last_os_error())
        } else {
            Ok(read as usize)
        }
    }
}

impl Read for Fd {
    #[inline]
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        self.recv(buf)
    }
}

impl Write for Fd {
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

impl Opener for Fd {
    fn open(req: &req::IfReq) -> Result<Self> {
        let fd = unsafe { libc::open(PATH.as_ptr() as *const libc::c_char, libc::O_RDWR) };
        if fd < 0 {
            return Err(Error::FS {
                path: unsafe { String::from_utf8_unchecked(PATH.to_vec()) },
                source: io::Error::last_os_error(),
            });
        }

        let ret = unsafe { create_queue(fd, req as *const req::IfReq as PointerWidth)? };
        if ret >= 1 {
            return Err(Error::from(ret as i32));
        }
        Ok(Self(fd))
    }
}

impl Closer for Fd {
    fn close(&mut self) -> Result<()> {
        let ret = unsafe { libc::close(self.0) };
        if ret < 0 {
            Err(Error::errno())
        } else {
            self.0 = -1;
            Ok(())
        }
    }
}
