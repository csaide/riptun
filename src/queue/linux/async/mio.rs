// (c) Copyright 2021 Christian Saide
// SPDX-License-Identifier: MIT

use super::Queue;

use std::io;
use std::os::unix::io::AsRawFd;

use mio::unix::SourceFd;
use mio::{event, Interest, Registry, Token};

impl event::Source for Queue {
    fn register(
        &mut self,
        registry: &Registry,
        token: Token,
        interests: Interest,
    ) -> io::Result<()> {
        SourceFd(&self.as_raw_fd()).register(registry, token, interests)
    }

    fn reregister(
        &mut self,
        registry: &Registry,
        token: Token,
        interests: Interest,
    ) -> io::Result<()> {
        SourceFd(&self.as_raw_fd()).reregister(registry, token, interests)
    }

    fn deregister(&mut self, registry: &Registry) -> io::Result<()> {
        SourceFd(&self.as_raw_fd()).deregister(registry)
    }
}
