// (c) Copyright 2021 Christian Saide
// SPDX-License-Identifier: MIT

use riptun::{AsyncStdTun, Result};

use std::io::ErrorKind;

const NUM_QUEUES: usize = 5;

#[async_std::main]
pub async fn main() -> Result<()> {
    // Create a new async Tun using the AsyncStdTun implementation, named `rip%d` with
    // NUM_QUEUES internal queues.
    let async_dev = AsyncStdTun::new("rip%d", NUM_QUEUES)?;

    // Print out the OS given name of this device.
    println!("[INFO] => Created new virtual device: {}", async_dev.name());

    // Create a buffer the same size as the MTU as the Tun device, which by default
    // is 1500 Bytes.
    let mut buffer: [u8; 1500] = [0x00; 1500];

    // Loop forever reading packets off the Tun.
    loop {
        // Calling [AsyncStdTun::recv()] handles selecting an available queue and reading one
        // packet off the selected queue. See the documentation for further details on its internal
        // selection mechanism.
        let read = match async_dev.recv(&mut buffer).await {
            Ok(read) => read,
            // If its a WouldBlock simply short circuit, and re-poll the AsyncStdQueue.
            Err(err) if err.kind() == ErrorKind::WouldBlock => continue,
            Err(err) => {
                // If we error simply log the issue, and continue on business as usual.
                println!("[ERROR] => {}", err);
                continue;
            }
        };

        // Print the packet data as a byte slice, noting its size.
        println!("[INFO] => Packet data ({}B): {:?}", read, &buffer[..read]);
    }
}
