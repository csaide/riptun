# (c) Copyright 2021 Christian Saide
# SPDX-License-Identifier: MIT

###
# OS Determination
###

ifeq '$(findstring ;,$(PATH))' ';'
    detected_OS := Windows
else
    detected_OS := $(shell uname 2>/dev/null || echo Unknown)
    detected_OS := $(patsubst CYGWIN%,Cygwin,$(detected_OS))
    detected_OS := $(patsubst MSYS%,MSYS,$(detected_OS))
    detected_OS := $(patsubst MINGW%,MSYS,$(detected_OS))
endif

ifeq ($(detected_OS),Windows)
    DEFAULT_OS := windows
endif
ifeq ($(detected_OS),Darwin)
    DEFAULT_OS := darwin
endif
ifeq ($(detected_OS),Linux)
    DEFAULT_OS := linux
endif
ifeq ($(detected_OS),FreeBSD)
    DEFAULT_OS := freebsd
endif
ifeq ($(detected_OS),NetBSD)
    DEFAULT_OS := netbsd
endif


###
# Build args and variables.
###

BUILD                   := debug
HASH                    := $(shell git rev-parse HEAD)
DEFAULT_ARCH            := amd64
DEFAULT_TARGET          := $(DEFAULT_OS)-$(DEFAULT_ARCH)
DEFAULT_EXAMPLES_ACTION := examples.$(DEFAULT_TARGET)
DEFAULT_COMPILE_ACTION  := compile.$(DEFAULT_TARGET)

###
# Target and build definitions for varios OS/arch combinations
###

# Define the resulting targets for building cross-platform
target_linux-arm     = arm-unknown-linux-gnueabihf
target_linux-armv7   = armv7-unknown-linux-gnueabihf
target_linux-arm64   = aarch64-unknown-linux-gnu
target_linux-amd64   = x86_64-unknown-linux-gnu
target_freebsd-amd64 = x86_64-unknown-freebsd
target_netbsd-amd64  = x86_64-unknown-netbsd
target_windows-amd64 = x86_64-pc-windows-msvc
target_windows-arm64 = aarch64-pc-windows-msvc
target_darwin-amd64  = x86_64-apple-darwin
target_darwin-arm64  = aarch64-apple-darwin

# Define an override so that we can turn on/off release builds.
build_debug =
build_release = --release

###
# Default target definition
###

.PHONY: default devel
default: full
devel: check full

###
# Binary compilation steps.
###

.PHONY: docs compile examples full compile.linux compile.windows compile.darwin examples.linux examples.windows examples.darwin examples.netbsd examples.freebsd

docs:
	@bash ./dist/bin/print.sh "Generating Docs"
	@cargo doc --no-deps

examples.%:
	@bash ./dist/bin/print.sh "Building examples: '$*' mode: '$(BUILD)'"
	@cargo build --target $(target_$*) --features async-mio-fd --example mio
	@cargo build --target $(target_$*) --features async-smol-fd-example --example smol
	@cargo build --target $(target_$*) --features async-std-fd-example --example std
	@cargo build --target $(target_$*) --features async-tokio-fd-example --example tokio
	@cargo build --target $(target_$*) --example sync

examples.linux: \
	examples.linux-amd64 \
	examples.linux-arm64 \
	examples.linux-armv7 \
	examples.linux-arm

examples.windows: \
	examples.windows-amd64 \
	examples.windows-arm64

examples.darwin: \
	examples.darwin-amd64 \
	examples.darwin-arm64

examples.netbsd: \
	examples.netbsd-amd64

examples.freebsd: \
	examples.netbsd-amd64

examples: examples.$(DEFAULT_OS)

compile.%:
	@bash ./dist/bin/print.sh "Building target: '$*' mode: '$(BUILD)'"
	@cargo build $(build_$(BUILD)) --target $(target_$*)

compile.linux: \
	compile.linux-amd64 \
	compile.linux-arm64 \
	compile.linux-armv7 \
	compile.linux-arm

compile.windows: \
	compile.windows-amd64 \
	compile.windows-arm64

compile.darwin: \
	compile.darwin-amd64 \
	compile.darwin-arm64

compile.netbsd: \
	compile.netbsd-amd64

compile.freebsd: \
	compile.freebsd-amd64

compile: compile.$(DEFAULT_OS)

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

coverage:
	@bash ./dist/bin/print.sh "Running tests with coverage"
	@mkdir -p target/coverage/
	@cargo tarpaulin  -o Html --output-dir target/coverage/

license:
	@bash ./dist/bin/print.sh "Verifying licensing"
	@bash ./dist/bin/lic-check.sh

check: fmt lint units license

###
# Package/Publish
###

.PHONY: package publish

login:
	@bash ./dist/bin/print.sh "Authorizing Cargo"
	@cargo login $(CARGO_API_KEY)

package:
	@bash ./dist/bin/print.sh "Packaging"
	@cargo package --allow-dirty

publish:
	@bash ./dist/bin/print.sh "Packaging"
	@cargo publish

###
# Cleanup
###

.PHONY: clean

clean:
	@bash ./dist/bin/print.sh "Cleaning"
	@rm -rf \
		target/doc \
		target/debug \
		target/release \
		target/coverage \
		target/criterion \
		target/tarpaulin \
		target/package \
		target/arm-unknown-linux-gnueabihf \
		target/x86_64-unknown-linux-gnu \
		target/aarch64-unknown-linux-gnu \
		target/armv7-unknown-linux-gnueabihf
