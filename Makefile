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
ifeq ($(detected_OS),OpenBSD)
    DEFAULT_OS := openbsd
endif


###
# Build args and variables.
###

.SECONDEXPANSION:
BUILD := debug

###
# Target and build definitions for varios OS/arch combinations
###

# Define the resulting targets for building cross-platform
target_linux := \
	arm-unknown-linux-gnueabi \
	arm-unknown-linux-gnueabihf \
	armv7-unknown-linux-gnueabi \
	armv7-unknown-linux-gnueabihf \
	aarch64-unknown-linux-gnu \
	x86_64-unknown-linux-gnu
target_freebsd := \
	x86_64-unknown-freebsd \
	aarch64-unknown-freebsd
target_netbsd := \
	x86_64-unknown-netbsd \
	aarch64-unknown-netbsd
target_openbsd := \
	x86_64-unknown-openbsd \
	aarch64-unknown-openbsd
target_windows := \
	x86_64-pc-windows-msvc \
	aarch64-pc-windows-msvc
target_darwin := \
	x86_64-apple-darwin \
	aarch64-apple-darwin

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

.PHONY: docs compile examples full
.PHONY: compile.linux compile.windows compile.darwin compile.openbsd compile.netbsd compile.freebsd
.PHONY: examples.linux examples.windows examples.darwin examples.openbsd examples.netbsd examples.freebsd

docs:
	@bash ./dist/bin/print.sh "Generating Docs"
	@cargo doc

# Ensure we build each example with the appropriate limited set of features.
examples-bin.%:
	@bash ./dist/bin/print.sh "Building examples: '$*' mode: '$(BUILD)'"
	@cargo build $(build_$(BUILD)) --target $* --no-default-features --features mio-impl --example mio
	@cargo build $(build_$(BUILD)) --target $* --no-default-features --features smol-example --example smol
	@cargo build $(build_$(BUILD)) --target $* --no-default-features --features async-std-example --example std
	@cargo build $(build_$(BUILD)) --target $* --no-default-features --features tokio-example --example tokio
	@cargo build $(build_$(BUILD)) --target $* --no-default-features --example sync

# Build all targets for the given OS.
examples-exp.%: $$(foreach target,$$(target_$$*),examples-bin.$$(target))
	@bash ./dist/bin/print.sh "Finished building examples for OS: '$*' mode: '$(BUILD)'"

# By default build examples for the local OS, but in theory it should be possible to at least
# compile cross-platform.
examples.linux:   examples-exp.linux
examples.windows: examples-exp.windows
examples.darwin:  examples-exp.darwin
examples.openbsd: examples-exp.openbsd
examples.netbsd:  examples-exp.netbsd
examples.freebsd: examples-exp.freebsd
examples: examples.$(DEFAULT_OS)

# Ensure we compile each of the targets properly using the correct mode.
compile-bin.%:
	@bash ./dist/bin/print.sh "Building target: '$*' mode: '$(BUILD)'"
	@cargo build $(build_$(BUILD)) --target $*

# Build all targets for the biven OS.
compile-exp.%: $$(foreach target,$$(target_$$*),compile-bin.$$(target))
	@bash ./dist/bin/print.sh "Finished building targets for OS: '$*' mode: '$(BUILD)'"

# By default build targets for the local OS, but in theory it should be possible to at least
# compile cross-platform.
compile.linux: compile-exp.linux
compile.windows: compile-exp.windows
compile.darwin: compile-exp.darwin
compile.openbsd: compile-exp.openbsd
compile.netbsd: compile-exp.netbsd
compile.freebsd: compile-exp.freebsd
compile: compile.$(DEFAULT_OS)

# Default action to compile all targets + examples.
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
	@cargo clippy -- --no-deps

units:
	@bash ./dist/bin/print.sh "Running tests"
	@cargo test

coverage:
	@bash ./dist/bin/print.sh "Running tests with coverage"
	@mkdir -p target/coverage/
	@cargo tarpaulin -o Html --output-dir target/coverage/

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
	@bash ./dist/bin/print.sh "Publishing"
	@cargo publish

publish-ci:
	@bash ./dist/bin/print.sh "Publishing"
	@cargo --token $(CARGO_API_KEY) publish

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
		target/arm-unknown-linux-gnueabi \
		target/x86_64-unknown-linux-gnu \
		target/aarch64-unknown-linux-gnu \
		target/armv7-unknown-linux-gnueabihf \
		target/armv7-unknown-linux-gnueabi
