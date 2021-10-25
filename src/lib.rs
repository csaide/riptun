// (c) Copyright 2021 Christian Saide
// SPDX-License-Identifier: MIT

#![warn(missing_docs)]

//! riptun is a library for creating, managing, and leveraging TUN/TAP devices.

mod error;
#[cfg_attr(target_os = "linux", path = "linux/mod.rs")]
mod queue;

pub use error::{Error, Result};
pub use queue::{Fd, Tun};

#[cfg(feature = "async-std-fd")]
pub use queue::{AsyncStdFd, AsyncStdTun};

#[cfg(feature = "async-tokio-fd")]
pub use queue::{TokioFd, TokioTun};
