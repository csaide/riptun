// (c) Copyright 2021 Christian Saide
// SPDX-License-Identifier: MIT

use riptun::{Queue, Tun};

use std::collections::HashMap;
use std::error::Error;

use mio::{Events, Interest, Poll, Token};

const NUM_QUEUES: usize = 5;

fn main() -> Result<(), Box<dyn Error>> {
    let mut poll = Poll::new()?;
    let mut events = Events::with_capacity(128);

    let (mut sync, name) = match Tun::new("rip%d", NUM_QUEUES) {
        Ok(sync) => sync,
        Err(err) => return Err(Box::new(err)),
    };

    println!("[INFO] => Created new virtual device: {}", name);

    let mut queues: HashMap<Token, Queue> = HashMap::with_capacity(NUM_QUEUES);
    for (idx, mut queue) in sync.drain(..).enumerate() {
        queue.set_non_blocking(true)?;
        let token = Token(idx);
        poll.registry()
            .register(&mut queue, token, Interest::READABLE | Interest::WRITABLE)?;
        queues.insert(token, queue);
    }

    // Start an event loop.
    loop {
        // Poll Mio for events, blocking until we get an event.
        poll.poll(&mut events, None)?;

        let mut buffer: [u8; 65535] = [0x00; 65535];

        // Process each event.
        for event in events.iter() {
            let queue = match queues.get(&event.token()) {
                Some(queue) => queue,
                None => {
                    println!("[WARN] Unexpected token found: {:?}", event.token());
                    continue;
                }
            };
            if event.is_readable() {
                let read = match queue.recv(&mut buffer) {
                    Ok(read) => read,
                    Err(err) => {
                        println!("[ERROR][Queue: {:?}] => {}", event.token(), err);
                        continue;
                    }
                };
                println!(
                    "[INFO][Queue: {:?}] => Packet data ({}B): {:?}",
                    event.token(),
                    read,
                    &buffer[..read]
                );
            }
        }
    }
}
