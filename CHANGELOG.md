# Change Log

Dose Response is roguelike game (currently in development) where you
play an addict. Avoid the dangers threatening your mind and body while
desperately looking for the next fix.

## [Unreleased]
- Mouse support in the Help screen


## [0.6.0] - 2018-02-16
- Added saving and loading
- Fixed fullscreen (including on multi-monitor setups)
- Fixed food appearing on the player's initial position
- Fixed doses' irresistible areas covering the player's position
- There's now at least one clear safe-to-use dose nearby
- Basic mouse support in the main menu
- Increased the rate at which the player develops tolerance


## [0.5.2] - 2018-01-29
- Added Help screen (press `?` to open it)
- Added Main menu (press `Esc` to open it)
- Added variable-width text rendering (used in the menus and help)


## [0.5.1] - 2018-01-11
- Added hints to the end game screen
- Fixed the Vi keys


## [0.5.0] - 2017-12-30
- Toggle fullscreen when pressing Alt+Enter
- NPCs give bonuses when bumped into while sober
  - The player can only have 1 bonus active at a time
  - The bonus goes away if the player gets High again
- WebAssembly support!
  - The game can now be played (in a modern up-to-date browser) at:
  - https://aimlesslygoingforward.com/projects/dose-response-roguelike/play.html
- Different ways of losing the game now fade in different colours


## [0.4.3] - 2017-04-28

- Make sure the player is not immediately surrounded by monsters
- Ditto for being in the range of an irresistible dose
- Introduce monsters gradually. At first, only the `S` and `v` will
  appear and only once the player veers from the starting area will
  they encounter the rest.
- Add useless NPCs (they can't be interacted within, but they're
  friendly and just wander around)
- Make the `h` monsters follow a pack-like behaviour: if one sees a
  player, it tells other `h`s nearby
- Show keys for starting a new game and quitting in the endgame screen
- Doses' effect only contributes to the intoxicated state (i.e.
  whether or how *sober* or *exhausted* you are makes no effect while
  using)
- When winning the game, show the endgame screen
- The endgame screens shows why the player lost t
- The Windows version no longer shows the command line console


## [0.4.2] - 2017-03-18

- The shattering doses actually show up in the inventory
- Weaker doses can be picked up earlier than on maximum Will
- The keys for movements are shown every new game (and disappear when you get
  close to them so as not to interfere)


## [0.4.1] - 2017-02-26

- Made the shattering doses destroy the environment


## [0.4.0] - 2017-02-25

- The game world is now effectively infinite
- Added "vi keys": you can use h/j/k/l to move left/up/down/right and
  y/n/u/m to move north-west/south-west/north-east/south-east
- Idle monsters now pick a destination to walk to instead of just
  randomly moving to a surrounding tile
- After losing, a summary with the total number of turns, longest High
  streak and number of doses in your inventory is shown
- Two new dose types are added. Their strength is between the existing
  two types, but they shoot cardinal or diagonal rays that destroy
  walls


## [0.3.0] - 2016-12-14

- The game can be won now (see Spoilers for more details)
- You can pick doses up and carry them your inventory when your will
  is at its maximum (5 in this release)
- Press keys 1, 2 or 3 to use food/dose/strong dose from your
  inventory
- Doses appear more frequently on the map
- More detailed display of the state of the mind: a textual
  representation (withdrawn/sober/high) as well as a progress bar
  showing how fare along a given state you are
- When the dose's effects disappear, you go directly to "withdrawn",
  skipping the "sober" state. You can work your way up to sobe by
  eating food
- Different background colour for the sidebar so it doesn't blend into
  the unexplored area
- Fix screen scrolling - the screen now scrolls when the field of view
  would touch the edge of the screen


### Spoilers

To win the game, you must defeat enough a monsters to increase your
Will to the maximum value (5). You need to defeat 7 as to increase the
Will by one point. Once you're there, you need to last for 100 turns
without getting high, losing will or (of course) dying.


[Unreleased]: https://github.com/tomassedovic/dose-response/compare/v0.4.3...HEAD
[0.4.3]: https://github.com/tomassedovic/dose-response/compare/v0.4.2...v0.4.3
[0.4.2]: https://github.com/tomassedovic/dose-response/compare/v0.4.1...v0.4.2
[0.4.1]: https://github.com/tomassedovic/dose-response/compare/v0.4.0...v0.4.1
[0.4.0]: https://github.com/tomassedovic/dose-response/compare/v0.3.0...v0.4.0
