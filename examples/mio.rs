// (c) Copyright 2021 Christian Saide
// SPDX-License-Identifier: MIT

use riptun::{Queue, Tun};

use std::collections::HashMap;
use std::io::{self, ErrorKind};
use std::time::Duration;

use mio::{Events, Interest, Poll, Token};

const NUM_QUEUES: usize = 5;

fn main() -> io::Result<()> {
    // Create a new synchronous Tun named `rip%d` with NUM_QUEUES internal queues.
    let mut sync = Tun::new("rip%d", NUM_QUEUES).map_err(|err| err.into_io())?;

    // Print out the OS given name of this device.
    println!("[INFO] => Created new virtual device: {}", sync.name());

    // Create a new Poll instance so that we can listen for events on our Tun Queues.
    let mut poll = Poll::new()?;

    // Drain the Tun instance of its internal Queue instances, taking ownership and allowing
    // for full lifecycle management. This allows us to feed the Queue's into the mio registry
    // and handle the Token > Queue mapping.
    let mut queues: HashMap<Token, Queue> = HashMap::with_capacity(NUM_QUEUES);
    for (idx, mut queue) in sync.drain(..).enumerate() {
        // Ensure the Queue's underlying file descriptor is in non-blocking mode.
        queue.set_non_blocking(true).map_err(|err| err.into_io())?;

        // Create this queue's token, register, and then map this queue to that
        // generated token.
        let token = Token(idx);
        poll.registry()
            .register(&mut queue, token, Interest::READABLE)?;
        queues.insert(token, queue);
    }

    // Create the event storage and start the event loop.
    let mut events = Events::with_capacity(NUM_QUEUES);

    // Create a buffer the same size as the MTU as the Tun device, which by default
    // is 1500 Bytes.
    let mut buffer: [u8; 1500] = [0x00; 1500];
    loop {
        // Poll Mio for events, blocking until we get an event or 500ms elapses.
        poll.poll(&mut events, Some(Duration::from_millis(500)))?;

        // Lets short circuit if we don't have any events.
        if events.is_empty() {
            continue;
        }

        // Process each event.
        for event in events.iter() {
            // For the purposes of this example we are only interested in read insterests.
            if event.is_readable() {
                // Grab the queue from our map so we can use it to read the next packet.
                let queue = queues.get(&event.token()).unwrap();

                // Actually call recv on the Queue itself, which will return the number
                // of bytes actually read into the supplied buffer.
                let read = match queue.recv(&mut buffer) {
                    Ok(read) => read,
                    // If its a WouldBlock simply short circuit, and re-poll the Queues.
                    Err(err) if err.kind() == ErrorKind::WouldBlock => continue,
                    Err(err) => {
                        // If we error simply log the issue, and continue on business as usual.
                        println!("[ERROR][Queue: {:?}] => {}", event.token(), err);
                        continue;
                    }
                };

                // Print the packet data as a byte slice, noting its size and which queue the
                // packet came from.
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
