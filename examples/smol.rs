// (c) Copyright 2021 Christian Saide
// SPDX-License-Identifier: MIT

use riptun::AsyncStdTun;

const NUM_QUEUES: usize = 5;

pub async fn run() {
    let mut async_dev = match AsyncStdTun::new("rip%d", NUM_QUEUES) {
        Ok(async_dev) => async_dev,
        Err(err) => {
            println!("[ERROR] => {}", err);
            return;
        }
    };

    println!("[INFO] => Created new virtual device: {}", async_dev.name());

    let mut handles = Vec::with_capacity(NUM_QUEUES);
    for (idx, queue) in async_dev.drain(..).enumerate() {
        let handle = smol::spawn(async move {
            let mut buffer: [u8; 1500] = [0x00; 1500];
            loop {
                let read = match queue.recv(&mut buffer).await {
                    Ok(read) => read,
                    Err(err) => {
                        println!("[ERROR][Queue: {}] => {}", idx, err);
                        return;
                    }
                };
                println!(
                    "[INFO][Queue: {}] => Packet data ({}B): {:?}",
                    idx,
                    read,
                    &buffer[..read]
                );
            }
        });
        handles.push(handle);
    }
    futures_util::future::join_all(handles).await;
}

pub fn main() {
    smol::block_on(async { run().await })
}
