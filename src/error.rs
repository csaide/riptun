// (c) Copyright 2021 Christian Saide
// SPDX-License-Identifier: MIT

// stdlib usings
use std::{io, result};

// extern usings
use thiserror::Error;

/// Custom Result wrapper to simplify usage.
pub type Result<T> = result::Result<T, Error>;

#[derive(Error, Debug)]
/// Represents store errors based on user configuration or
/// general operations.
pub enum Error {
    /// Internal raw unix error.
    #[error("fatal unix error encountered: {source}")]
    Unix {
        /// The internal unix error encountered.
        source: nix::errno::Errno,
    },
    /// Invalid ioctl return code encountered.
    #[error("ioctl failed with unexpected return code: got '{0}'")]
    UnixIoctl(i32),
    /// Internal file I/O errors encountered.
    #[error("filesystem operation failed on '{path}': {source}")]
    FS {
        /// The path associated with this IO error.
        path: String,
        /// The underlying IO error kind.
        source: io::Error,
    },
    /// Internal generic I/O errors encountered.
    #[error("input/output operation failed")]
    IO {
        /// The internal IO error that occured.
        #[from]
        source: io::Error,
    },
    /// The specified queue descriptor is invalid.
    #[error("invalid queue descriptor specified '{0}' is out of range")]
    InvalidQueue(usize),
    /// The specified number of queues was less than or equal to 0.
    #[error("invalid number of queues specified must be greater than 0")]
    InvalidNumQueues,
    /// The specified device name is invalid.
    #[error(
        "invalid device name '{name}' is either longer than {max_size}B or the encoding is invalid"
    )]
    InvalidName {
        /// The supplied name.
        name: String,
        /// The max size of the name.
        max_size: usize,
    },
}

impl Error {
    /// Returns the last errno observed on unix systems.
    pub fn errno() -> Self {
        Self::from(nix::errno::Errno::last())
    }

    /// Consume this error and return the equivalent [std::io::Error].
    pub fn into_io(self) -> std::io::Error {
        use std::io::ErrorKind;
        match self {
            Self::FS { source, .. } => source,
            Self::IO { source } => source,
            Self::Unix { source } => std::io::Error::from_raw_os_error(source as i32),
            err => std::io::Error::new(ErrorKind::Other, err),
        }
    }
}

impl From<i32> for Error {
    fn from(i: i32) -> Self {
        Self::UnixIoctl(i)
    }
}

impl From<usize> for Error {
    fn from(i: usize) -> Self {
        Self::InvalidQueue(i)
    }
}

impl From<nix::errno::Errno> for Error {
    fn from(e: nix::errno::Errno) -> Self {
        Self::Unix { source: e }
    }
}
