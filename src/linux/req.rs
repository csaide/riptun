// (c) Copyright 2021 Christian Saide
// SPDX-License-Identifier: MIT

use nix::libc;

use super::{Error, Result};

const IF_NAME_SIZE: usize = libc::IFNAMSIZ;
const IFF_TUN: u16 = libc::IFF_TUN as u16;
const IFF_NO_PI: u16 = libc::IFF_NO_PI as u16;
const IFF_MULTI_QUEUE: u16 = libc::IFF_MULTI_QUEUE as u16;

#[derive(Debug)]
#[repr(C)]
pub struct IfReq {
    name: [u8; IF_NAME_SIZE],
    flags: u16,
}

impl IfReq {
    pub fn new(name_str: &str) -> Result<Self> {
        if name_str.is_empty() || !name_str.is_ascii() {
            return Err(Error::InvalidName {
                max_size: IF_NAME_SIZE,
                name: String::from(name_str),
            });
        }

        let mut name = [b'\0'; IF_NAME_SIZE];
        name_str
            .as_bytes()
            .iter()
            .take(IF_NAME_SIZE)
            .enumerate()
            .for_each(|(idx, char)| name[idx] = *char);

        let flags = IFF_TUN | IFF_NO_PI | IFF_MULTI_QUEUE;
        Ok(Self { name, flags })
    }

    pub fn name(&self) -> String {
        self.name
            .iter()
            .take_while(|char| **char != b'\0')
            .map(|char| *char as char)
            .collect::<String>()
    }
}
