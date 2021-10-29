// (c) Copyright 2021 Christian Saide
// SPDX-License-Identifier: MIT

#![warn(missing_docs)]

//! `riptun` is a library for creating, managing, and leveraging TUN/TAP devices.
//!
//! The implementation exposes both a synchronous interface via the [Tun] and [Queue] structs, as well as an
//! asynchronous interface via a set of feature flagged structs.
//!
//! ## Getting started
//!
//! The simplest way to get started with riptun is to manage a single queue synchronous TUN device:
//!
//! Lets start by disabling the async support as we won't be using it:
//!
//! ```toml
//! riptun = { version = "0.1", default-features = false, features = [] }
//! ```
//!
//! The following example program will create a new TUN device named `rip%d`, where the `%d`
//! will be replaced with an appropriate value by the OS. The exact device name along with the
//! actual TUN device is then returned for use. We then loop forever reading packets and printing
//! them to stdout:
//!
//! ```no_run
//! use riptun::Tun;
//!
//! // First lets create a new single queue tun.
//! let (tun, name) = match Tun::new("rip%d", 1) {
//!     Ok(tun) => tun,
//!     Err(err) => {
//!         println!("[ERROR] => {}", err);
//!         return;
//!     }
//! };
//!
//! // Lets make sure we print the real name of our new TUN device.
//! println!("[INFO] => Created TUN '{}'!", name);
//!
//! // Create a buffer to read packets into, and setup the queue to receive from.
//! let mut buffer: [u8; 1500] = [0x00; 1500];
//! let queue = 0;
//!
//! // Loop forever reading packets off the queue.
//! loop {
//!     // Receive the next packet from the specified queue.
//!     let read = match tun.recv_via(queue, &mut buffer) {
//!         Ok(read) => read,
//!         Err(err) => {
//!             println!("[ERROR] => {}", err);
//!             return;
//!         }
//!     };
//!
//!     // Print out the amount of data received and the bytes read off the queue.
//!     println!(
//!         "[INFO] => Received packet data ({}B): {:?}",
//!         read,
//!         &buffer[..read]
//!     );
//! }
//! ```
//!
//! Once the `rip%d` device is created, you will need to add an IP address to it. On Linux this can be
//! done like:
//!
//! ```no_compile
//! sudo ip addr add 203.0.113.2/24 brd 203.0.113.255 dev rip0
//! sudo ip link set dev rip0 up
//! ```
//!
//! # Features
//!
//! The async support is enabled by default, and `riptun` can be used out of the box across mio, tokio,
//! async-std, and smol. However to reduce library size, you can enable and disable each of the integrations
//! using feature flags:
//! - The `async-std-fd` feature exposes the [AsyncStdQueue]/[AsyncStdTun] structs.
//! - The `async-tokio-fd` feature exposes the [TokioQueue]/[TokioTun] structs.
//! - The `async-mio-fd` enables registration of [Queue] structs in a mio poll registry.
//!
//! # Platform support
//!
//! The `riptun` library is designed to be as platform agnostic as possible. Unfortunately each platform requires
//! special handling, so each platform must be implemented manually. The current and planned platform support
//! is detailed bellow.
//!
//! Platform/Architecture support matrix:
//!
//! | Target                          | Sync Supported | Async Supported |
//! |---------------------------------|:--------------:|:---------------:|
//! | `x86_64-unknown-linux-gnu`      | ✅              | ✅               |
//! | `aarch64-unknown-linux-gnu`     | ✅              | ✅               |
//! | `armv7-unknown-linux-gnueabihf` | ✅              | ✅               |
//! | `arm-unknown-linux-gnueabihf`   | ✅              | ✅               |
//! | `x86_64-pc-windows-msvc`        | ❌              | ❌               |
//! | `aarch64-pc-windows-msvc`       | ❌              | ❌               |
//! | `x86_64-apple-darwin`           | ❌              | ❌               |
//! | `aarch64-apple-darwin`          | ❌              | ❌               |
//! | `x86_64-unknown-freebsd`        | ❌              | ❌               |
//! | `x86_64-unknown-netbsd`         | ❌              | ❌               |
//!

mod error;
#[cfg_attr(target_os = "linux", path = "linux/mod.rs")]
mod queue;

pub use error::{Error, Result};
pub use queue::{Device, Queue, Tun};

#[cfg(feature = "async-std-fd")]
pub use queue::{AsyncStdQueue, AsyncStdTun};

#[cfg(feature = "async-tokio-fd")]
pub use queue::{TokioQueue, TokioTun};
