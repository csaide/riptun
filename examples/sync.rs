// (c) Copyright 2021 Christian Saide
// SPDX-License-Identifier: MIT

use riptun::{Result, Tun};

use std::sync::Arc;
use std::thread;

const NUM_QUEUES: usize = 5;

pub fn main() -> Result<()> {
    // Create a new synchronous Tun named `rip%d` with NUM_QUEUES internal queues.
    let sync = Tun::new("rip%d", NUM_QUEUES)?;

    // Print out the OS given name of this device.
    println!("[INFO] => Created new virtual device: {}", sync.name());

    // Create a Vec to store the JoinHandles in to eventually join over.
    let mut handles = Vec::with_capacity(NUM_QUEUES);

    // Since we need to clone the Tun across thread boundries lets wrap it in a Arc.
    let sync = Arc::new(sync);

    // Createa a new thread for each queue and have that thread execute forever reading packets
    // off its configured queue.
    for queue in 0..NUM_QUEUES {
        // Clone our Tun instance so that we can move it to the new thread.
        let handle_dev = sync.clone();

        // Spawn a new thread to execute for this queue.
        let handle = thread::spawn(move || {
            // Each thread gets its own buffer the same size as the MTU of the device, which by default
            // is 1500 bytes.
            let mut buffer: [u8; 1500] = [0x00; 1500];

            // Loop forever reading packets off the queue.
            loop {
                // Read the next packet off the specified queue and into the buffer blocking until data
                // is available to read.
                let read = match handle_dev.recv_via(queue, &mut buffer) {
                    Ok(read) => read,
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

        // Store the JoinHandle in our Vec.
        handles.push(handle);
    }

    // Loop over all handles Joining and noting any errors that come up.
    for handle in handles {
        if let Err(err) = handle.join() {
            println!("[ERROR] => {:?}", err);
        }
    }
    Ok(())
}
