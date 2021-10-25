// (c) Copyright 2021 Christian Saide
// SPDX-License-Identifier: MIT

use std::{sync::Arc, thread};

use riptun::Dev;

const NUM_QUEUES: usize = 5;

pub fn main() {
    let (async_dev, name) = match Dev::new("rip%d", NUM_QUEUES) {
        Ok(async_dev) => async_dev,
        Err(err) => {
            println!("[ERROR] => {}", err);
            return;
        }
    };

    println!("[INFO] => Created new virtual device: {}", name);

    let mut handles = Vec::with_capacity(NUM_QUEUES);
    let async_dev = Arc::new(async_dev);
    for queue in 0..NUM_QUEUES {
        let handle_dev = async_dev.clone();
        let handle = thread::spawn(move || {
            let mut buffer: [u8; 65535] = [0x00; 65535];
            loop {
                let read = match handle_dev.recv(queue, &mut buffer) {
                    Ok(read) => read,
                    Err(err) => {
                        println!("[ERROR][Queue: {}] => {}", queue, err);
                        return;
                    }
                };
                println!(
                    "[INFO][Queue: {}] => Packet data ({}B): {:?}",
                    queue,
                    read,
                    &buffer[..read]
                );
            }
        });
        handles.push(handle);
    }
    for handle in handles {
        if let Err(err) = handle.join() {
            println!("[ERROR] => {:?}", err);
        }
    }
}
