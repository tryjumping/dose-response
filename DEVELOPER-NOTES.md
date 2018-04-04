Dose Response Developer Notes
=============================

Building
--------

Requires Rust 1.26.

For the SDL backend (planned to become default) you also need the SDL2
library available wherever your OS looks for libraries.


Headless / Remote-controlled Mode
---------------------------------

Dose Response can be controlled remotely via ZeroMQ. This is mostly
for testing and it's disabled by default.

To compile it in you need to have zeromq-devel (or equivalent) installed.

Build it with:

    cargo build --features=remote

And then pass `--remote` to the `dose-response` executable.


Webassembly
-----------

Install the wasm toolchain:

    rustup update nightly
    rustup target add wasm32-unknown-unknown --toolchain=nightly

Compile:

    cargo +nightly build --release --target wasm32-unknown-unknown --no-default-features

The compiled file will be created in: `target/wasm32-unknown-unknown/release/dose-response.wasm`

NOTE: while the code compiles, running it will panic on calling the
random number generator. Until that works, working on wasm is not
particularly useful.
