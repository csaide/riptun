// (c) Copyright 2021 Christian Saide
// SPDX-License-Identifier: MIT

use super::{Error, Result};

use cfg_if::cfg_if;

mod req;
mod sync;

use req::IfReq;
pub use sync::Queue;

pub(crate) fn new_queues<T>(name: &str, num_queues: usize) -> Result<(Vec<T>, String)>
where
    T: Opener,
{
    let req = IfReq::new(name)?;
    let mut queues = Vec::with_capacity(num_queues);
    for _ in 0..num_queues {
        let queue = T::open(&req)?;
        queues.push(queue);
    }
    Ok((queues, req.name()))
}

pub(crate) trait Opener: Sized {
    fn open(req: &IfReq) -> Result<Self>;
}

cfg_if! {
    if #[cfg(feature = "async-std-impl")] {
        #[path = "async/std.rs"]
        mod async_std;
        pub use self::async_std::AsyncStdQueue;
    }
}

cfg_if! {
    if #[cfg(feature = "tokio-impl")] {
        #[path = "async/tokio.rs"]
        mod async_tokio;
        pub use self::async_tokio::TokioQueue;
    }
}

cfg_if! {
    if #[cfg(feature = "mio-impl")] {
        #[path = "async/mio.rs"]
        pub mod async_mio;
    }
}
