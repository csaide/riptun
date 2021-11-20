// (c) Copyright 2021 Christian Saide
// SPDX-License-Identifier: MIT

use riptun::{Result, TokioTun};

use std::io::ErrorKind;
use std::sync::Arc;

const NUM_QUEUES: usize = 5;

#[tokio::main]
pub async fn main() -> Result<()> {
    // Create a new async Tun using the TokioTun implementation, named `rip%d` with
    // NUM_QUEUES internal queues.
    let async_dev = TokioTun::new("rip%d", NUM_QUEUES)?;

    // Print out the OS given name of this device.
    println!("[INFO] => Created new virtual device: {}", async_dev.name());

    // Create a Vec to store the Futures in to eventually join on.
    let mut handles = Vec::with_capacity(NUM_QUEUES);

    // Since we need to clone the TokioTun across thread boundries lets wrap it in a Arc.
    let async_dev = Arc::new(async_dev);

    // Createa a new Future for each queue and have that Future execute forever reading packets
    // off its configured queue.
    for queue in 0..NUM_QUEUES {
        // Clone our TokioTun instance so that we can move it to the new thread.
        let handle_dev = async_dev.clone();

        // Spawn a new Future to execute for this queue.
        let handle = tokio::spawn(async move {
            // Each Future gets its own buffer the same size as the MTU of the device, which by default
            // is 1500 bytes.
            let mut buffer: [u8; 1500] = [0x00; 1500];

            // Loop forever reading packets off the queue.
            loop {
                // Read the next packet off the specified queue and into the buffer using the async/await
                // syntax to execute the future returned by recv_via.
                let read = match handle_dev.recv_via(queue, &mut buffer).await {
                    Ok(read) => read,
                    // If its a WouldBlock simply short circuit, and re-poll the AsyncStdQueue.
                    Err(err) if err.kind() == ErrorKind::WouldBlock => continue,
                    Err(err) => {
                        // If we error simply log the issue, and continue on business as usual.
                        println!("[ERROR][Queue: {:?}] => {}", queue, err);
                        continue;
                    }
                };

                // Print the packet data as a byte slice, noting its size and which queue the
                // packet came from.
                println!(
                    "[INFO][Queue: {}] => Packet data ({}B): {:?}",
                    queue,
                    read,
                    &buffer[..read]
                );
            }
        });

        // Store the future in our Vec.
        handles.push(handle);
    }

    // Now execute all of the Futures we generated above and await for them to finish. Note that in this program
    // this line will never return.
    futures_util::future::join_all(handles).await;

    Ok(())
}
