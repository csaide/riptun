// (c) Copyright 2021 Christian Saide
// SPDX-License-Identifier: MIT

use super::{Error, Result};

use cfg_if::cfg_if;

mod dev;
mod fd;
mod req;

use dev::Device;
pub use dev::Tun;
pub use fd::Fd;
use req::IfReq;

pub trait Opener: Sized {
    fn open(req: &IfReq) -> Result<Self>;
}

pub trait Closer: Sized {
    fn close(&mut self) -> Result<()>;
}

cfg_if! {
    if #[cfg(feature = "async-std-fd")] {
        #[path = "async_fd/std.rs"]
        mod async_std_fd;
        #[path = "async_dev/std.rs"]
        mod async_std_dev;

        pub use async_std_dev::AsyncStdTun;
        pub use async_std_fd::AsyncStdFd;
    }
}

cfg_if! {
    if #[cfg(feature = "async-tokio-fd")] {
        #[path = "async_fd/tokio.rs"]
        mod async_tokio_fd;
        #[path = "async_dev/tokio.rs"]
        mod async_tokio_dev;

        pub use async_tokio_dev::TokioTun;
        pub use async_tokio_fd::TokioFd;
    }
}

cfg_if! {
    if #[cfg(feature = "async-mio-fd")] {
        #[path = "async_fd/mio.rs"]
        pub mod async_fd;
    }
}
