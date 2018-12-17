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

    $ cargo build --no-default-features --features 'cli replay rand fullscreen cheating stats verifications glium-backend'

To run the built binary, you must pass the `--glium` command line argument:

    $ ./target/debug/dose-response --glium

Webassembly
-----------

Install the wasm toolchain:

    rustup update stable
    rustup target add wasm32-unknown-unknown

Compile:

    cargo build --release --target wasm32-unknown-unknown --no-default-features --features web

The compiled file will be created in: `target/wasm32-unknown-unknown/release/dose-response.wasm`

You can test it by serving the contents of `web-src`. Something like
this should work if you've got `python3` installed:

    $ cd dose-response/web-src
    $ python3 -m http.server
    Serving HTTP on 0.0.0.0 port 8000 (http://0.0.0.0:8000/) ...

Now open [http://0.0.0.0:8000/](http://0.0.0.0:8000/) in your web
browser.


Recording a video
-----------------

The game is able to save all the frames as images on disk. This can be
used to "record" a gameplay video.

Currently, only the glium backend is able to do this. In addition,
it's veeeery slow and you really want to do this in the release mode
(debug records about 1 frame per second). To produce the final video,
you'll want to have `ffmpeg` installed.

Steps:

1. Install ffmpeg
2. `mkdir /home/thomas/tmp/dose-response-recording`
   * yep, it's hardcoded
3. `cargo build --release`
4. `cargo run -- --glium --record`
   * replays work as well
5. `cd /home/thomas/tmp/dose-response-recording; ls`
6. `ffmpeg -framerate 60 -i "img%06d.png" output.mp4`

You can also use a containerised `ffmpeg` if you want:

    podman run -v $PWD:/out:z --rm -i jrottenberg/ffmpeg -framerate 60 -i "/out/img%06d.png" /out/output.mp4


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
[glium]: https://crates.io/crates/glium
[glutin]: https://crates.io/crates/glutin
