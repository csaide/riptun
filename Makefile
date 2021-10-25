# (c) Copyright 2021 Christian Saide
# SPDX-License-Identifier: MIT

###
# Build args and variables.
###

BUILD := debug
HASH := $(shell git rev-parse HEAD)
DEFAULT_OS := linux
DEFAULT_ARCH := amd64
DEFAULT_TARGET := $(DEFAULT_OS)-$(DEFAULT_ARCH)
DEFAULT_EXAMPLES_ACTION := examples.$(DEFAULT_TARGET)
DEFAULT_COMPILE_ACTION := compile.$(DEFAULT_TARGET)

###
# Target and build definitions for varios OS/arch combinations
###

# Define the resulting targets for building cross-platform
target_linux-arm = armv7-unknown-linux-gnueabihf
target_linux-arm64 = aarch64-unknown-linux-gnu
target_linux-amd64 = x86_64-unknown-linux-gnu
# TODO(csaide): Disabled for now need to fix cross platform builds.
#
# target_windows-amd64 = x86_64-pc-windows-msvc
# target_windows-arm64 = aarch64-pc-windows-msvc
# target_darwin-amd64 = x86_64-apple-darwin
# target_darwin-arm64 = aarch64-apple-darwin

# Define an override so that we can turn on/off release builds.
build_debug =
build_release = --release

###
# Default target definition
###

.PHONY: default devel
default: $(DEFAULT_COMPILE_ACTION) $(DEFAULT_EXAMPLES_ACTION)
devel: check $(DEFAULT_COMPILE_ACTION) $(DEFAULT_EXAMPLES_ACTION)

###
# Binary compilation steps.
###

.PHONY: docs compile examples full

docs:
	@bash ./dist/bin/print.sh "Generating Docs"
	@cargo doc --no-deps --document-private-items

examples.%:
	@bash ./dist/bin/print.sh "Building examples: '$*' mode: '$(BUILD)'"
	@cargo build --target $(target_$*) --features async-mio-fd --example mio
	@cargo build --target $(target_$*) --features async-smol-fd-example --example smol
	@cargo build --target $(target_$*) --features async-std-fd-example --example std
	@cargo build --target $(target_$*) --features async-tokio-fd-example --example tokio
	@cargo build --target $(target_$*) --example sync

examples: \
	examples.linux-amd64 \
	examples.linux-arm64 \
	examples.linux-arm
# TODO(csaide): Disabled for now need to fix cross platform TLS builds.
#
# examples.windows-amd64 \
# examples.windows-arm64 \
# examples.darwin-amd64 \
# examples.darwin-arm64

compile.%:
	@bash ./dist/bin/print.sh "Building target: '$*' mode: '$(BUILD)'"
	@cargo build $(build_$(BUILD)) --target $(target_$*)

compile: \
	compile.linux-amd64 \
	compile.linux-arm64 \
	compile.linux-arm
# TODO(csaide): Disabled for now need to fix cross platform TLS builds.
#
# compile.windows-amd64 \
# compile.windows-arm64 \
# compile.darwin-amd64 \
# compile.darwin-arm64

full: compile examples

###
# Source code validation, formatting, linting.
###

.PHONY: fmt lint units bench coverage license check

fmt:
	@bash ./dist/bin/print.sh "Formatting Code"
	@cargo fmt --all -- --emit=files

lint:
	@bash ./dist/bin/print.sh "Linting"
	@cargo fmt --all -- --check
	@cargo clippy --target $(target_$(DEFAULT_TARGET)) -- --no-deps

units:
	@bash ./dist/bin/print.sh "Running tests"
	@cargo test --target $(target_$(DEFAULT_TARGET))

bench:
	@bash ./dist/bin/print.sh "Running benchmarks"
	@cargo bench --target $(target_$(DEFAULT_TARGET))

coverage:
	@bash ./dist/bin/print.sh "Running tests with coverage"
	@mkdir -p target/coverage/
	@cargo tarpaulin  -o Html --output-dir target/coverage/

license:
	@bash ./dist/bin/print.sh "Verifying licensing"
	@bash ./dist/bin/lic-check.sh

check: fmt lint units bench license

###
# Cleanup
###

.PHONY: clean

clean:
	@bash ./dist/bin/print.sh "Cleaning"
	@rm -rf \
		target/linux-arm64 \
		target/linux-amd64 \
		target/linux-arm \
		target/doc \
		target/debug \
		target/release \
		target/coverage \
		target/criterion \
		target/tarpaulin \
		target/x86_64-unknown-linux-gnu \
		target/aarch64-unknown-linux-gnu \
		target/armv7-unknown-linux-gnueabihf
