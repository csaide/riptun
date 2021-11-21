# riptun
[![linux](https://github.com/csaide/riptun/actions/workflows/linux.yml/badge.svg)](https://github.com/csaide/riptun/actions/workflows/linux.yml)
[![codecov](https://codecov.io/gh/csaide/riptun/branch/develop/graph/badge.svg?token=JIJN96Q4RG)](https://codecov.io/gh/csaide/riptun)
[![version](https://img.shields.io/crates/v/riptun)](https://crates.io/crates/riptun)
[![license](https://img.shields.io/crates/l/riptun)](https://github.com/tokio-rs/tokio/blob/master/LICENSE)

`riptun` is a library for creating, managing, and leveraging TUN/TAP devices.

The implementation exposes both a synchronous interface via the [Tun] and [Queue] structs, as well as an
asynchronous interface via a set of feature flagged structs. See the [Features](#features) and [Examples](#examples)
sections for more information on the async implementations and how to use them.

# Getting started

The simplest way to get started with riptun is to manage a single queue synchronous TUN device:

Lets start by disabling the async support as we won't be using it:

```toml
riptun = { version = "0.1", default-features = false, features = [] }
```

The following example program will create a new TUN device named `rip%d`, where the `%d`
will be replaced with an appropriate value by the OS. The exact device name along with the
actual TUN device is then returned for use. We then loop forever reading packets and printing
them to stdout:

```no_run
use riptun::Tun;

// First lets create a new single queue tun.
let tun = match Tun::new("rip%d", 1) {
    Ok(tun) => tun,
    Err(err) => {
        println!("[ERROR] => {}", err);
        return;
    }
};

// Lets make sure we print the real name of our new TUN device.
println!("[INFO] => Created TUN '{}'!", tun.name());

// Create a buffer to read packets into, and setup the queue to receive from.
let mut buffer: [u8; 1500] = [0x00; 1500];
let queue = 0;

// Loop forever reading packets off the queue.
loop {
    // Receive the next packet from the specified queue.
    let read = match tun.recv_via(queue, &mut buffer) {
        Ok(read) => read,
        Err(err) => {
            println!("[ERROR] => {}", err);
            return;
        }
    };

    // Print out the amount of data received and the bytes read off the queue.
    println!(
        "[INFO] => Received packet data ({}B): {:?}",
        read,
        &buffer[..read]
    );
}
```

Once the `rip%d` device is created, you will need to add an IP address to it. On Linux this can be
done like:

```bash
sudo ip addr add 203.0.113.2/24 brd 203.0.113.255 dev rip0
sudo ip link set dev rip0 up
```

# Examples

There is a suite of included examples demonstrating the functionality of `riptun`. Note that the following examples
will require elevated privileges to configure and create the actual Tun interface itself. This generally means `root`
or `Administrator` privileges across unix and windows platforms.

See the [examples directory](examples/) for more information on the available example programs.

# Development

The `riptun` library is developed with the following tools:
- GNU Make
- Rust 1.53+
- Docker (linux only)

Make is used to facilitate simple and easy build/test/packaging commands. The Makefile will automatically target the local OS
that is being used to build `riptun` and will also automatically build all configured targets for said platform. This means that
in order to build and develop `riptun` across platforms is simply having `rust` and the requisite C compilers available for the
configured targets.

## Compilation

To compile on the local OS run the following:

```bash
$ make
```

Which will compile all local OS targets, and compile all examples in `debug` mode. In order to build in `release` mode
run the following:

```bash
$ make BUILD=release
```

## Testing

Testing `riptun` requires administrator privileges, this is due to the fact that creating virtual interfaces across OSes requires
this level of privileges.

### Linux

In order to run the end to end testing suite on Linux run the following:

```bash
# First ensure the `/dev/net/tun` virtual file exists.
$ sudo bash dist/bin/create-tun.sh
# Then go ahead and run the test suite.
$ sudo make check
```

### Windows

> TODO(csaide): Implement windows support.

### Darwin

> TODO(csaide): Implement darwin support.

### BSD

> TODO(csaide): Implement open/net/freebsd support.

# Features

The async support is enabled by default, and `riptun` can be used out of the box across mio, tokio,
async-std, and smol. However to reduce library size, you can enable and disable each of the integrations
using feature flags:
- The `async-std-impl` feature exposes the [AsyncStdQueue]/[AsyncStdTun] structs.
- The `tokio-impl` feature exposes the [TokioQueue]/[TokioTun] structs.
- The `mio-impl` enables registration of [Queue] structs in a mio poll registry.

# Platform support

The `riptun` library is designed to be as platform agnostic as possible. Unfortunately each platform requires
special handling, so each platform must be implemented manually. The current and planned platform support
is detailed bellow.

Platform/Architecture support matrix:

| Target                          | Sync Supported | Async Supported |
|---------------------------------|:--------------:|:---------------:|
| `x86_64-unknown-linux-gnu`      | ✅              | ✅               |
| `aarch64-unknown-linux-gnu`     | ✅              | ✅               |
| `armv7-unknown-linux-gnueabihf` | ✅              | ✅               |
| `armv7-unknown-linux-gnueabi`   | ✅              | ✅               |
| `arm-unknown-linux-gnueabihf`   | ✅              | ✅               |
| `arm-unknown-linux-gnueabi`     | ✅              | ✅               |
| `x86_64-pc-windows-msvc`        | ❌              | ❌               |
| `aarch64-pc-windows-msvc`       | ❌              | ❌               |
| `x86_64-apple-darwin`           | ❌              | ❌               |
| `aarch64-apple-darwin`          | ❌              | ❌               |
| `x86_64-unknown-freebsd`        | ❌              | ❌               |
| `aarch64-unknown-freebsd`       | ❌              | ❌               |
| `x86_64-unknown-netbsd`         | ❌              | ❌               |
| `aarch64-unknown-netbsd`        | ❌              | ❌               |
| `x86_64-unknown-openbsd`        | ❌              | ❌               |
| `aarch64-unknown-openbsd`       | ❌              | ❌               |

---
&copy; Copyright 2021 Christian Saide
