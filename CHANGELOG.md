# Change Log

Dose Response is roguelike game (currently in development) where you
play an addict. Avoid the dangers threatening your mind and body while
desperately looking for the next fix.

## [Unreleased]

[Unreleased]: https://github.com/tryjumping/dose-response/compare/v0.11.0...HEAD

## [0.11.1] - 2018-12-14
- Updated all in-game links to use the new https://tryjumping.com website
- Added icon to the Windows executables
- Migrated code repository to https://github.com/tryjumping/dose-response
- Added counter showing progress on increasing the Will stat
- Companion NPCs are now always visible -- even when behind a fog of war or unexplored tile
- Save files now store the game version and log a warning if it doesn't match
- The game now shows the main menu on start
  - Previously, the player was dumped straight into the game

[0.11.1]: https://github.com/tryjumping/dose-response/compare/v0.10.0...v0.11.1


## [0.10.0] - 2018-11-30
- Fixed crash when restarting the game in the browser
- Fixed a visual glitch when scrolling the map
- Victory NPCs leave an explanation when they leave because the player got high
- Companion NPCs now move as fast as the player and can't be easily left behind
- Added an About page to the in-game help
- Added version, homepage and git commit to the the game log (dose-response.log)
- Added version to the main menu
- Fixed the display size in the glium backend

[0.10.0]: https://github.com/tryjumping/dose-response/compare/v0.9.0...v0.10.0

## [0.9.0] - 2018-11-03
- Fixed the dose ordering in the sidebar
- Updated the help messages
- Player and monsters now alternate their turns
  - this is only really visible if the player has more than 1 Action Point
  - previously, the player would use up all their APs and then monsters would do the same
  - now, the player uses 1 AP, then the monsters use 1 and so on
- Added Victory NPCs
  - This is the proper endgame sequence
  - Once the player can resist all doses, a new NPC will spawn
  - Upon reaching it, the game ends
- Show a path to the Victory NPC when they appear
- The camera briefly scrolls to Victory NPC when they appear
- Removed the "sobriety counter" placeholder endgame sequence
- Ported the code to Rust 2018 edition

[0.9.0]: https://github.com/tryjumping/dose-response/compare/v0.8.4...v0.9.0


## [0.8.4] - 2018-06-8
- Defaulted to the SDL backend everywhere
- Fixed the OSX build
- Added logging to the dose-response.log file

[0.8.4]: https://github.com/tryjumping/dose-response/compare/v0.8.3...v0.8.4


## [0.8.3] - 2018-05-15
- Added OpenGL implementation for the SDL backend
- Lots of internal code changes

[0.8.3]: https://github.com/tryjumping/dose-response/compare/v0.8.2...v0.8.3


## [0.8.2] - 2018-05-05
- Minor build script fixes

[0.8.2]: https://github.com/tryjumping/dose-response/compare/v0.8.1...v0.8.2


## [0.8.1] - 2018-05-05
- Minor build script fixes

[0.8.1]: https://github.com/tryjumping/dose-response/compare/v0.8.0...v0.8.1


## [0.8.0] - 2018-05-05
- The monsters now have names visible in the game
- Described how to use items in the help pages
- Fixed glyph position in the opengl backend
- Fixed the spacing between paragraphs in the help pages
- Increased the font size
- Fixed mouse position issues on fullscreen
- Increased the AoE radius of the Diagonal and Cardinal doses
- Added a What's Dose Response help page
- Added smooth scrolling when centering the screen
- The player can now move even when an animation is playing
- Switched the web version to use WebGL

[0.8.0]: https://github.com/tryjumping/dose-response/compare/v0.7.0...v0.8.0


## [0.7.0] - 2018-03-10
- Added mouse support to the web version
- Items can be used with mouse now
- All menus have mouse support
- You can close windows by right-clicking anywhere on the screen
- Made the colours a bit nicer
- Disabled Fullscreen under Windows because it crashes the game
- Removed fog-of-war at the endgame screen
- Included Readme and License files with the game

[0.7.0]: https://github.com/tryjumping/dose-response/compare/v0.6.0...v0.7.0


## [0.6.0] - 2018-02-16
- Added saving and loading
- Fixed fullscreen (including on multi-monitor setups)
- Fixed food appearing on the player's initial position
- Fixed doses' irresistible areas covering the player's position
- There's now at least one clear safe-to-use dose nearby
- Basic mouse support in the main menu
- Increased the rate at which the player develops tolerance

[0.6.0]: https://github.com/tryjumping/dose-response/compare/v0.5.2...v0.6.0


## [0.5.2] - 2018-01-29
- Added Help screen (press `?` to open it)
- Added Main menu (press `Esc` to open it)
- Added variable-width text rendering (used in the menus and help)

[0.5.2]: https://github.com/tryjumping/dose-response/compare/v0.5.1...v0.5.2


## [0.5.1] - 2018-01-11
- Added hints to the end game screen
- Fixed the Vi keys

[0.5.1]: https://github.com/tryjumping/dose-response/compare/v0.5.0...v0.5.1


## [0.5.0] - 2017-12-30
- Toggle fullscreen when pressing Alt+Enter
- NPCs give bonuses when bumped into while sober
  - The player can only have 1 bonus active at a time
  - The bonus goes away if the player gets High again
- WebAssembly support!
  - The game can now be played (in a modern up-to-date browser) at:
  - https://tryjumping.com/dose-response-roguelike/play/
- Different ways of losing the game now fade in different colours

[0.5.0]: https://github.com/tryjumping/dose-response/compare/v0.4.3...v0.5.0


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

[0.4.3]: https://github.com/tryjumping/dose-response/compare/v0.4.2...v0.4.3


## [0.4.2] - 2017-03-18

- The shattering doses actually show up in the inventory
- Weaker doses can be picked up earlier than on maximum Will
- The keys for movements are shown every new game (and disappear when you get
  close to them so as not to interfere)

[0.4.2]: https://github.com/tryjumping/dose-response/compare/v0.4.1...v0.4.2


## [0.4.1] - 2017-02-26

- Made the shattering doses destroy the environment

[0.4.1]: https://github.com/tryjumping/dose-response/compare/v0.4.0...v0.4.1


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

[0.4.0]: https://github.com/tryjumping/dose-response/compare/v0.3.0...v0.4.0


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
