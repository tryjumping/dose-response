Dose Response
=============

Dose Response is an unfinished roguelike where you play an addict.
Avoid the dangers threatening your mind and body while desperately
looking for the next fix.

How To Play
-----------

You control the `@` character on the screen. You can move in the four
**cardinal directions** (up, down, left and right) and four **diagonal
directions** (north-east, south-east, north-west, south-west).

Whenever you see a `%` it's a food that you can pick up and use later.
If you move onto it, you pick it up automatically.

`i` is a dose and your character uses it as soon as you step on it. It
destroys nearby monsters and expands your field of view.

When your come down from the effect of the dose, you'll go into a
withdrawal. Eventually, you'll lose of exhaustion.

You can eat food (or find another dose) to stave off the effects.

When you have an item you can use (such as food) it will appear in
your inventory in the sidebar next to a numeric key. You use it by
pressing the key.


Remember:
* At the beginning, you can overdose very easily
* Doses differ in purity, sometimes you come across a weak one and
  other times it's the bomb
* You build up tolerance so you'll eventually have to consume more and
  more doses or move onto the stronger stuff
* It is possible to win the game

## Controls

There is no mouse support (yet!) in the game so you play by pressing
the keys. There are three keyboard schemes you can use:

### Arrows + `ctrl`/`shift`

Use your `up`, `down`, `left` and `right` keys to move
up/down/left/right.

To move diagonally, pressing `control` and `left`/`right` means "north"
and pressing `shift` means "south".

So to move north-east you hold `control` and press `right`. To go
south-west you hold `shift` and press `left`.

### Numpad

Most desktop computers have a numpad, but a lot of laptops don't. If
you have it, you can use its keys to move around:

Use `8`, `2`, `4` and `6` keys to move up/down/left/right.

To move diagonally, use `7`, `9`, `1` and `3` to move north-west,
north-east, south-west and south-east.

### Vi-keys

Use `k`, `j`, `h` and `l` to move up/down/left/right.

To move diagonally, press `y`, `n`, `u` and `m` to move north-west,
south-west, north-east and south-west.



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

License
-------

Copyright (C) 2016 Tomas Sedovic

This program is free software: you can redistribute it and/or modify
it under the terms of the GNU General Public License as published by
the Free Software Foundation, either version 3 of the License, or
(at your option) any later version.

This program is distributed in the hope that it will be useful,
but WITHOUT ANY WARRANTY; without even the implied warranty of
MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
GNU General Public License for more details.

You should have received a copy of the GNU General Public License
along with this program.  If not, see <http://www.gnu.org/licenses/>.
