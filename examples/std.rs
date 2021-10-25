// (c) Copyright 2021 Christian Saide
// SPDX-License-Identifier: MIT

use riptun::AsyncStdTun;

const NUM_QUEUES: usize = 5;

#[async_std::main]
pub async fn main() {
    let (async_dev, name) = match AsyncStdTun::new("rip%d", NUM_QUEUES) {
        Ok(async_dev) => async_dev,
        Err(err) => {
            println!("[ERROR] => {}", err);
            return;
        }
    };

    println!("[INFO] => Created new virtual device: {}", name);

    let mut buffer: [u8; 65535] = [0x00; 65535];
    loop {
        let read = match async_dev.recv(&mut buffer).await {
            Ok(read) => read,
            Err(err) => {
                println!("[ERROR] => {}", err);
                return;
            }
        };
        println!("[INFO] => Packet data ({}B): {:?}", read, &buffer[..read]);
    }
}
