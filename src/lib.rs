// (c) Copyright 2021 Christian Saide
// SPDX-License-Identifier: MIT

#![warn(missing_docs)]

//! `riptun` is a library for creating, managing, and leveraging TUN/TAP devices.
//!
//! The implementation exposes both a synchronous interface via the [Tun] and [Queue] structs, as well as an
//! asynchronous interface via a set of feature flagged structs. See the [Features](#features) and [Examples](#examples)
//! sections for more information on the async implementations and how to use them.
//!
//! # Getting started
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
//! let tun = match Tun::new("rip%d", 1) {
//!     Ok(tun) => tun,
//!     Err(err) => {
//!         println!("[ERROR] => {}", err);
//!         return;
//!     }
//! };
//!
//! // Lets make sure we print the real name of our new TUN device.
//! println!("[INFO] => Created TUN '{}'!", tun.name());
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
//! ```bash
//! sudo ip addr add 203.0.113.2/24 brd 203.0.113.255 dev rip0
//! sudo ip link set dev rip0 up
//! ```
//!
//! # Examples
//!
//! There is a suite of included examples demonstrating the functionality of `riptun`. Note that the following examples
//! will require elevated privileges to configure and create the actual Tun interface itself. This generally means `root`
//! or `Administrator` privileges across unix and windows platforms.
//!
//! As in the [Getting Started](#getting-started) section above, all the examples will require assigning an IP address to the
//! created interface, you can do so by running:
//!
//! ```bash
//! sudo ip addr add 203.0.113.2/24 brd 203.0.113.255 dev rip0
//! sudo ip link set dev rip0 up
//! ```
//!
//! ## Sync
//!
//! The synchronous mode of operation is detailed in the [sync example](https://github.com/csaide/riptun/blob/master/examples/sync.rs). The
//! implementation leverages a multi-queue [Tun] device. Then creates a new thread for each queue, and loops forever reading packets from
//! each one.
//!
//! You can run this example using the following and get similar output if you have the right permissions:
//!
//! ```bash
//! cargo run --no-default-features --example sync
//! [INFO] => Created new virtual device: rip0
//! ```
//!
//! ## Mio
//!
//! The mio integration allows for registering and deregistering [Queue] instances with a Poll Registry. The [mio example](https://github.com/csaide/riptun/blob/master/examples/mio.rs)
//! shows how to leverage this integration for a multi-threaded async version of the [Sync](#sync) example above.
//!
//! You can run this example using the following and get similar output if you have the right permissions:
//!
//! ```bash
//! cargo run --no-default-features --features mio-impl --example mio
//! [INFO] => Created new virtual device: rip0
//! ```
//!
//! ## Smol and async-std
//!
//! `riptun` exposes an [async-io](https://docs.rs/async-io/1.6.0/async_io/index.html) integration via the [AsyncStdTun] and [AsyncStdQueue] structs. These can be used interchangeably
//! across both [smol](https://github.com/smol-rs/smol) and [async-std](https://docs.rs/async-std/1.10.0/async_std/index.html).
//!
//! The [std example](https://github.com/csaide/riptun/blob/master/examples/std.rs) describes how to leverage a multi-queue device using a single thread of operation. While using the `async-std` runtime.
//!
//! You can run the `std` example using the following and get similar output if you have the right permissions:
//!
//! ```bash
//! cargo run --no-default-features --features async-std-example --example std
//! [INFO] => Created new virtual device: rip0
//! ```
//!
//! The [smol example](https://github.com/csaide/riptun/blob/master/examples/smol.rs) describes how to leverage a multi-queue device using a single thread, but with multiple concurrent tasks. While using the `smol` runtime.
//!
//! You can run the `smol` example using the following and get similar output if you have the right permissions:
//!
//! ```bash
//! cargo run --no-default-features --features smol-example --example smol
//! [INFO] => Created new virtual device: rip0
//! ```
//!
//! ## Tokio
//!
//! The `riptun` tokio integration is exposed via the [TokioTun] and [TokioQueue] structs. The [tokio example](https://github.com/csaide/riptun/blob/master/examples/tokio.rs) demonstrates
//! how to leverage both multi-threaded and concurrent operation simultaneously.
//!
//! You can run this example using the following and get similar output if you have the right permissions:
//!
//! ```bash
//! cargo run --no-default-features --features tokio-example --example tokio
//! [INFO] => Created new virtual device: rip0
//! ```
//!
//! # Features
//!
//! The async support is enabled by default, and `riptun` can be used out of the box across mio, tokio,
//! async-std, and smol. However to reduce library size, you can enable and disable each of the integrations
//! using feature flags:
//! - The `async-std-impl` feature exposes the [AsyncStdQueue]/[AsyncStdTun] structs.
//! - The `tokio-impl` feature exposes the [TokioQueue]/[TokioTun] structs.
//! - The `mio-impl` enables registration of [Queue] structs in a mio poll registry.
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

use cfg_if::cfg_if;

mod error;
#[cfg_attr(target_os = "linux", path = "queue/linux/mod.rs")]
mod queue;
mod tun;

pub use error::{Error, Result};
pub use queue::Queue;
pub use tun::Tun;

cfg_if! {
    if #[cfg(feature = "async-std-impl")] {
        pub use queue::AsyncStdQueue;
        pub use tun::AsyncStdTun;
    }
}

cfg_if! {
    if #[cfg(feature = "tokio-impl")] {
        pub use queue::TokioQueue;
        pub use tun::TokioTun;
    }
}
