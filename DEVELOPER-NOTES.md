Dose Response Developer Notes
=============================

Building
--------

Requires Rust 1.31 (the code uses the [Rust 2018 edition][edition]).

For the SDL backend (planned to become default) you also need
the [SDL2][sdl] library available wherever your OS looks for
libraries.


Pure Rust
---------

By default, Dose Response uses [SDL2][sdl] as the graphics backend. This
requires having the SDL2 libraries installed on your system.

If you want to try the pure Rust graphics backend
([winit][winit], [glutin][glutin] & [glium][glium]), compile the
project like so:

    $ cargo build --no-default-features --features 'cli replay rand fullscreen cheating stats verifications opengl'

To run the built binary, you must pass the `--opengl` command line argument:

    $ ./target/debug/dose-response --opengl

Webassembly
-----------

Install the wasm toolchain:

    rustup update beta
    rustup target add wasm32-unknown-unknown --toolchain=beta

Compile:

    cargo +beta build --release --target wasm32-unknown-unknown --no-default-features

The compiled file will be created in: `target/wasm32-unknown-unknown/release/dose-response.wasm`

You can test it by serving the contents of `web-src`. Something like
this should work if you've got `python3` installed:

    $ cd dose-response/web-src
    $ python3 -m http.server
    Serving HTTP on 0.0.0.0 port 8000 (http://0.0.0.0:8000/) ...

Now open [http://0.0.0.0:8000/][http://0.0.0.0:8000/] in your web
browser.


Headless / Remote-controlled Mode
---------------------------------

Dose Response can be controlled remotely via ZeroMQ. This is mostly
for testing and it's disabled by default.

To compile it in you need to have zeromq-devel (or equivalent) installed.

Build it with:

    cargo build --features=remote

And then pass `--remote` to the `dose-response` executable.

[edition]: https://rust-lang-nursery.github.io/edition-guide/rust-2018/index.html
[sdl]: https://www.libsdl.org/
[winit]: https://crates.io/crates/winit
[glium]: https://crates.io/crates/glutin
[glutin]: https://crates.io/crates/glium
