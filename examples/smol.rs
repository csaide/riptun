// (c) Copyright 2021 Christian Saide
// SPDX-License-Identifier: MIT

use riptun::{AsyncStdTun, Result};

use std::io::ErrorKind;

const NUM_QUEUES: usize = 5;

pub async fn run() -> Result<()> {
    // Create a new async Tun using the AsyncStdTun implementation, named `rip%d` with
    // NUM_QUEUES internal queues.
    let mut async_dev = AsyncStdTun::new("rip%d", NUM_QUEUES)?;

    // Print out the OS given name of this device.
    println!("[INFO] => Created new virtual device: {}", async_dev.name());

    // Create a Vec to store the Futures in to eventually join on.
    let mut handles = Vec::with_capacity(NUM_QUEUES);

    // Drain the AsyncStdTun instance of its internal AsyncStdQueue instances, taking ownership and allowing
    // for full lifecycle management. This allows us to feed the AsyncStdQueue's into smol tasks, and individually
    // wait on each queue. This approach is perfect for multi-threaded executors, and will allow for both concurrent,
    // and parallel execution of queue recv/send operations.
    for (idx, queue) in async_dev.drain(..).enumerate() {
        // For each of the instances create a new Future.
        let handle = smol::spawn(async move {
            // Each future gets its own buffer the same size as the MTU of the device, which by default
            // is 1500 bytes.
            let mut buffer: [u8; 1500] = [0x00; 1500];

            // Loop forever reading packets off the queue.
            loop {
                // Read the next packet off the queue and into the buffer using async/await syntax to execute the
                // Future returned by recv.
                let read = match queue.recv(&mut buffer).await {
                    Ok(read) => read,
                    // If its a WouldBlock simply short circuit, and re-poll the AsyncStdQueue.
                    Err(err) if err.kind() == ErrorKind::WouldBlock => continue,
                    Err(err) => {
                        // If we error simply log the issue, and continue on business as usual.
                        println!("[ERROR][Queue: {:?}] => {}", idx, err);
                        continue;
                    }
                };

                // Print the packet data as a byte slice, noting its size and which queue the
                // packet came from.
                println!(
                    "[INFO][Queue: {}] => Packet data ({}B): {:?}",
                    idx,
                    read,
                    &buffer[..read]
                );
            }
        });

        // Store the handle in our Vec.
        handles.push(handle);
    }

    // Now execute all of the Futures we generated above and await for them to finish. Note that in this program
    // this line will never return.
    futures_util::future::join_all(handles).await;

    Ok(())
}

pub fn main() -> Result<()> {
    smol::block_on(async { run().await })
}
