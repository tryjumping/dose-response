Dose Response
=============

Dose Response is an open-world roguelike where you play an addict.
Avoid the dangers threatening your mind and body while desperately
looking for the next fix.

Building The Code
-----------------

Have a look at the `DEVELOPER-NOTES.md` file in this repo or:

https://github.com/tryjumping/dose-response/blob/master/DEVELOPER-NOTES.md


How To Play
-----------

You control the `@` character on the screen. You can move in the four
**cardinal directions** (up, down, left and right) and four **diagonal
directions** (north-east, south-east, north-west, south-west).

For in-game help click on the `Help` button or press `?` while playing.

`%` represents food that you can pick up and use later.
Stepping into food will pick it up automatically.

`i` is a dose and the player consumes it when stepping on it. It
destroys nearby monsters and expands your field of view.

When coming off a dose high, you'll enter into a withdrawal state.
In this state exhaustion starts increasing, too much exhaustion and you lose.

You can eat food (or find another dose) to stave off the effects of withdrawal.

When you have an item you can use it will appear in
your inventory in the sidebar next to a numeric key. You use it by
pressing the corresponding key.


Quick Tips:
* At the beginning, you can overdose very easily
* Doses differ in purity, sometimes you come across a weak one and
  other times it's the bomb
* You build up tolerance so you'll eventually have to consume more and
  more doses or move onto the stronger stuff
* It is possible to win the game

## Controls

The game is controlled by keyboard. There are three keyboard schemes
you can use:

### Numpad (recommended)

The Numpad is the simplest control scheme to understand, each key
corresponds to its 8-way movement counterpart.

Use `8`, `2`, `4` and `6` keys to move up, down, left or right.

To move diagonally, use `7`, `9`, `1` and `3` to move north-west,
north-east, south-west and south-east.

If you have a laptop or a keyboard without a numpad the following
keyboard schemes are available.

### Arrows keys + `ctrl`/`shift`

Use your `up`, `down`, `left` and `right` keys to move
in those directions.

To move diagonally, pressing `control` and `left`/`right` means "north"
and pressing `shift` means "south".

So to move north-east you hold `control` and press `right`. To go
south-west you hold `shift` and press `left`.

### Vi-keys

Use `k`, `j`, `h` and `l` to move up, down, left or right.

To move diagonally, press `y`, `b`, `u` and `n` to move north-west,
south-west, north-east and south-west.




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
