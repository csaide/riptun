// (c) Copyright 2021 Christian Saide
// SPDX-License-Identifier: MIT

use super::{queue::new_queues, Error, Queue, Result};

use cfg_if::cfg_if;

mod sync;

pub use sync::Tun;

cfg_if! {
    if #[cfg(feature = "async-std-impl")] {
        use super::AsyncStdQueue;

        #[path = "async/std.rs"]
        mod async_std;
        pub use self::async_std::AsyncStdTun;
    }
}

cfg_if! {
    if #[cfg(feature = "tokio-impl")] {
        use super::TokioQueue;

        #[path = "async/tokio.rs"]
        mod async_tokio;
        pub use self::async_tokio::TokioTun;
    }
}
