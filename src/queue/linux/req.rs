// (c) Copyright 2021 Christian Saide
// SPDX-License-Identifier: MIT

use nix::libc;

use super::{Error, Result};

const IF_NAME_SIZE: usize = libc::IFNAMSIZ;
const IFF_TUN: u16 = libc::IFF_TUN as u16;
const IFF_NO_PI: u16 = libc::IFF_NO_PI as u16;
const IFF_MULTI_QUEUE: u16 = libc::IFF_MULTI_QUEUE as u16;
const IFF_FLAGS: u16 = IFF_TUN | IFF_NO_PI | IFF_MULTI_QUEUE;

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

        Ok(Self {
            name,
            flags: IFF_FLAGS,
        })
    }

    pub fn name(&self) -> String {
        self.name
            .iter()
            .take_while(|char| **char != b'\0')
            .map(|char| *char as char)
            .collect::<String>()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empy_name() {
        let req = IfReq::new("");
        assert!(req.is_err());
        match req.unwrap_err() {
            Error::InvalidName { .. } => assert!(true),
            _ => assert!(false),
        }
    }

    #[test]
    fn test_utf_name() {
        let req = IfReq::new("😀");
        assert!(req.is_err());
        match req.unwrap_err() {
            Error::InvalidName { .. } => assert!(true),
            _ => assert!(false),
        }
    }

    #[test]
    fn test_long_name() {
        let input = "aaaaaaaaaaaaaaaaaaaaaaaa";
        let expected = "aaaaaaaaaaaaaaaa";
        let req = IfReq::new(input);
        assert!(req.is_ok());
        let req = req.unwrap();
        assert_eq!(IFF_FLAGS, req.flags);
        assert_eq!(expected, req.name());
    }

    #[test]
    fn test_happy_path() {
        let req = IfReq::new("rip%d");
        assert!(req.is_ok());
        let req = req.unwrap();
        assert_eq!(IFF_FLAGS, req.flags);
        assert_eq!("rip%d", req.name());
    }
}
