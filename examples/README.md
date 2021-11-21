# Examples

The `riptun` library has a suite of examples showing off how to leverage the various implementations that the library exports.

The following examples exist
- [mio](#mio) demonstrates how to leverage a Poll registry with a `Tun` and its `Queue` instances.
- [smol](#smol) demonstrates how to leverage a `AsyncStdTun` and its `AsyncStdQueue` instances in the `smol` ecosystem.
- [std](#std) demonstrates how to leverage a `AsyncStdTun` and its `AsyncStdQueue` instances in the `async-std` ecosystem.
- [sync](#sync) demonstrates how to leverage a `Tun` using a multi-threaded approach.
- [tokio](#tokio) demonstrates how to leverage a `TokioTun` and its `TokioQueue` instances in the `tokio` ecosystem.

Currently `riptun` does not manage the networking configuration for virtual devices so once you run one of the above examples you will need to create a address for the link and then ensure the device is `up`. Once you see the log output as shown in the various examples bellow, run the appropriate commands, for your OS, to assign your new device an address. You should then be able to send traffic to the newly created device using `ping` or your favorite network diagnostic tool.

### Linux

On linux this can be accomplished with:

```bash
sudo ip addr add 203.0.113.2/24 brd 203.0.113.255 dev rip0
sudo ip link set dev rip0 up
```

### Windows

> TODO(csaide): Implement windows support.

### Darwin

> TODO(csaide): Implement darwin support.

### BSD

> TODO(csaide): Implement open/net/freebsd support.

## Mio

The mio example is a single threaded non-blocking mode `Tun` example. To run this example use:

```bash
$ sudo cargo run --no-default-features --features mio-impl --example mio
[INFO] => Created new virtual device: rip0
```

## Smol

The `smol` example leverages the `smol` executor in conjunction with the `AsyncStdTun` and `AsyncStdQueue` implementation. It then creates a separate async Future for each of the configured queues. And concurrently polls all queues simultaneously. To run this example use:

```bash
$ sudo cargo run --no-default-features --features smol-example --example smol
[INFO] => Created new virtual device: rip0
```

## Std

The `std` example leverages the `async-std` executor in conjunction with the `AsyncStdTun` and `AsyncStdQueue` implementation. It then leverages the exposed `recv` method on the `AsyncStdTun` to automatically handle polling all available queues for readiness. To run this example use:

```bash
$ sudo cargo run --no-default-features --features async-std-example --example std
[INFO] => Created new virtual device: rip0
```

## Sync

The `sync` example leverages the `std::thread` module in conjunction ith the `Tun` and `Queue` implementation. It then spawns a thread for each `Queue` and blocks on each of their respective `recv` calls. To run this example use:

```bash
$ sudo cargo run --no-default-features --example sync
[INFO] => Created new virtual device: rip0
```

## Tokio

The `tokio` example leverages the `tokio` ecosystem in conjunction with the `TokioTun` and `TokioQueue` implementation. It then creates a separate async Future for each of the configured queues. And concurrently polls all queues simultaneously. To run this example use:

```bash
$ sudo cargo run --no-default-features --features tokio-example --example tokio
[INFO] => Created new virtual device: rip0
```

---
&copy; Copyright 2021 Christian Saide
