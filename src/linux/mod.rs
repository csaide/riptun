// (c) Copyright 2021 Christian Saide
// SPDX-License-Identifier: MIT

use super::{Error, Result};

use cfg_if::cfg_if;

mod req;
mod sync_queue;
mod sync_tun;

use req::IfReq;
pub use sync_queue::Queue;
pub use sync_tun::{Device, Tun};

pub trait Opener: Sized {
    fn open(req: &IfReq) -> Result<Self>;
}

pub trait Closer: Sized {
    fn close(&mut self) -> Result<()>;
}

cfg_if! {
    if #[cfg(feature = "async-std-fd")] {
        #[path = "async_queue/std.rs"]
        mod async_std_queue;
        #[path = "async_dev/std.rs"]
        mod async_std_dev;

        pub use async_std_dev::AsyncStdTun;
        pub use async_std_queue::AsyncStdQueue;
    }
}

cfg_if! {
    if #[cfg(feature = "async-tokio-fd")] {
        #[path = "async_queue/tokio.rs"]
        mod async_tokio_queue;
        #[path = "async_dev/tokio.rs"]
        mod async_tokio_dev;

        pub use async_tokio_dev::TokioTun;
        pub use async_tokio_queue::TokioQueue;
    }
}

cfg_if! {
    if #[cfg(feature = "async-mio-fd")] {
        #[path = "async_queue/mio.rs"]
        pub mod async_queue;
    }
}
