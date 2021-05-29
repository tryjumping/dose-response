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
([winit][winit] & [glutin][glutin]), compile the
project like so:

    $ cargo build --no-default-features --features 'cli replay rand fullscreen cheating stats verifications glutin-backend'

To run the built binary, you must pass the `--glutin` command line argument:

    $ ./target/debug/dose-response --glutin

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


Publishing
----------

1. Update copyright notice (year) in README
2. Update copyright notice (year) in src/windows/help.rs
3. Update the version in Cargo.toml
4. Build the game

## Publishing on the web
1. Do the publishing steps
2. Run `make wasm-release`
3. `cp target/web/* path/to/website/play/`
4. Update the website (release numbers, copyright years)


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
4. `cargo run -- --glium --record-frames`
   * replays work as well
5. `cd /home/thomas/tmp/dose-response-recording; ls`
6. `ffmpeg -framerate 60 -i "img%06d.png" output.mp4`

You can also use a containerised `ffmpeg` if you want:

    podman run -v $PWD:/out:z --rm -i jrottenberg/ffmpeg -framerate 60 -i "/out/img%06d.png" /out/output.mp4

If you want to letterbox the video to a specific format, you can add this to ffmpeg:

    -vf "scale=(iw*sar)*min(1280/(iw*sar)\,720/ih):ih*min(1280/(iw*sar)\,720/ih), pad=1280:720:(1280-iw*min(1280/iw\,720/ih))/2:(720-ih*min(1280/iw\,720/ih))/2"

    -vf "scale=(iw*sar)*min(1280/(iw*sar)\,720/ih):ih*min(1280/(iw*sar)\,720/ih), pad=1280:720:(1280-iw*min(1280/iw\,720/ih)):(720-ih*min(1280/iw\,720/ih))/2"

The first option will centre the contents, the second one will move it
all the way to the right (under the assumption that the black bar is
looking better on the left-hand side where it can blend into the
unexplored area).

### Common Video dimensions

- 1920x1080 (1080p)
- 1280x720 (720p)
- 854x480 (480p)
- 640x360 (360p)

Settings for 720p:

* `height: f32 = 24.0` in `build.rs`
* `PANEL_WIDTH: i32 = 19` in `main.rs`



Adding messages into the replay log
-----------------------------------

If you want to pause a replay and show a message, you can put it in
the log manually. Timed message boxes have the following format:

    {"ShowMessageBox":{"ttl":{"secs":5,"nanos":6},"message":"Hello, world!"}}


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
