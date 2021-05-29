Dose Response Developer Notes
=============================

Building
--------

Requires Rust 1.31 (the code uses the [Rust 2018 edition][edition]).

The published release (versions 1.0 and 1.1) uses SDL2, but the master
(and future releases) uses pure Rust windowing libraries instead.


Dependencies
------------

The "alsa" development sources.

Fedora:

    # dnf install alsa-lib-devel

Debian/Ubuntu:

    # apt-get install libasound2-dev


Publishing
----------

1. Update copyright notice (year) in README
2. Update copyright notice (year) in src/windows/help.rs
3. Update the version in Cargo.toml
4. Build the game


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
