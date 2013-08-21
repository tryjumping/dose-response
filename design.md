Hedonic Hypothesis Roguelike: A design document
===============================================

"You cannot lose if you do not play." -- Marla Daniels

You're addicted. When you're not High, you're in a withdrawal. High doesn't last
long and the withdrawal is so bad you need to find another dose.

The law of diminishing returns can't be beat, though. Soon you're taking shots
just to keep you going (and you keep going to get more shots) while your fears,
anxiety and hunger descend on you and slowly wear you down.

If only you could get that magical elusive First High again. Everything would be
fixed, then.


Overview
--------

Hedonic Hypothesis is a Roguelike about addiction.

It is a roguelike in the following sense:
* Permadeath (when you die, you die. There's no save/load scumming)
* Randomly generated environment (every game has different surroundings)
* Text/character-based graphics and UI
* Hunger clock (the withdrawal mechanism forces you to keep going forward
  instead of exploring every single bit of the game)
* Turn-based

The game has a single character (displayed on the screen as the "at"
sign: @) controlled by the player. This character (PC) navigates a dungeon with
monsters (NPCs) and doses.

The PC is able to evade or slay the monsters and the monsters can kill the PC.
The substance give the PC a boost to their abilities but it steadily
deteriorates, forcing the PC to seek out more doses.

The environment ("dungeon") is split into an infinite number of randomly
generated floors. The player can descent deeper into the dungeon using stairs
that are always somewhere on the floor. They can always ascend back up, too but
they can't move past the first floor where the game began.

There is no winning condition. The player aims to last as long as they're able
before they suffer the inevitable end.


Technical notes
---------------

The game is rendered on a 80x25 Unicode character grid. It is possible to run it
from a dedicated window or within a terminal.

The game is controlled by keyboard only. The most oft-used keys are the keyboard
arrows, used to move the player around, navigate the menus, etc.


Screens
-------

The game has the following major screens:

1. Startup screen
2. New game screen
3. Load game screen
4. In-game screen
5. Main menu while running the game
6. The last playthrough statistics
7. Overall statistics


### Startup screen ###

Shows the game name and the (de-)motivational quota from Marla Daniels, followed
by the menu:

* Resume last game (PC's name + stats)
* New game
* Load game
* Character stats
* Quit

### Resume last game ###

Directly loads the saved game.

### New game screen ###

Asks you for a name.

No need to ask for the gender. The game is asexual -- it will avoid any text,
description, etc. that would prefer one sex over the other. I.e. the character's
sex is only in the player's head and need not be codified in the game in any
way.

<Esc> Back to main menu
<Enter> Start the game

### Load game screen ###

Numbered list of the saved games. Pressing the number will load the game.

<Esc> Back to main menu

### Character stats screen ###

Lists all the previous characters sorted by the amount of turns they survived.

<Esc> Back to main menu

### In-game main menu screen ###

Don't want to add the "Quit without saving" option. It risks losing players'
game by accident. We would have to add "are you sure? y/n" question which would
just complicate the UI.

Let's always save the progress on quit (actually, let's save it on every turn so
it's all right even when the game crashes). Storage is cheap.

If the player really wants to delete the save, they could do it from the Load
game menu.

* Quit (saves the game)
* Continue playing


In-game screen
--------------

Must show:
* Map
* Notification area
* PC attributes, State of Mind (textual: High, Withdrawal and graphical bar that
  goes down steadily)
* Floor number


Game rules and behaviours
-------------------------

0. The map is initially blank, the PC uncovers the dark areas by going there
1. The player character is represented by the `@` sign
2. The player moves the character in cardinal directions using the keyboard
arrows and the diagonal directions using numpad or <shift> and <ctrl> modifiers
3. The PC cannot move into or past the Walls
12. The PC has three main Attributes that can change over time: Confidence,
Nerve and Will
13. Each level has several Doses (displayed as `I`) randomly scattered across
the level
14. The PC can Use the Doses by bumping into them
18. The PC has a State of Mind property ranging from 0 to 100
19. State of Mind (need to find a better name for this) below 50 is considered
Withdrawal, above 60 is High, below 15 is Mindless (or Desperate).
20. Each Dose has four attributes: Strength, Confidence Boost, Nerve Boost and
Will Boost.
21. Using a Dose will make it disappear and will add the Boosts to the PC's
Attributes and the Strength to PC's State of Mind
22. The PC has a Tolerance attribute
23. Every time the PC uses a Dose, their Tolerance attribute increases. It never
decreases.
24. The Tolerance value decreases the gains from a used Dose -- the higher the
tolerance, the lower the gain
25. PC's State of Mind permanently deteriorates.
26. Once the State of Mind reaches zero, the PC dies of exhaustion.
27. When the State of Mind exceeds 100, the PC dies of overdose.
28. The game always starts with the PC at 20 State of Mind, in a room that's
empty apart from a Dose in sight.
30. The colour scheme of the surroundings responds dynamically to the State of
Mind: everything is bright and flashing when High and grows steadily bleaker and
grayer when getting Low.
29. The Doses are constantly displayed using bright colours. Their colour
doesn't correspond to the State of Mind, but since everything else does, the
Doses become more noticeable when in Withdrawal and blend in when High
30. Monsters move randomly on the screen
31. When the PC gets close to a monster, the monster targets the PC and starts
moving closer and eventually attacks
32. A character (PC or NPC) attacks its target by bumping into it
33. At the Desperate level of State of Mind, the PC loses control. The player
will no longer be able to control it and a very simple AI takes over: it will
lunge for the first Dose it detects and immediately consumes it.
34. At a low level of Courage, the PC gets into Panic: the player loses control
whenever a monster is in sight. The PC will run in an opposite direction.
35. Desperation overpowers Panic
36. The threshold for Desperation depends on the Will
37. The threshold for Panic depends on the Nerve
38.

4. The PC cannot move or see past Closed Doors
5. The PC can move into ("bump") the Closed Door to open it
6. The PC can move and see past Open Doors
7. The PC can descend into the level below by bumping into the Down Stairs
8. The PC can ascend into the level up by bumping into the Up Stairs
9. There are no Up Stairs on the first level
10. There are single Down Stairs on every level
11. There are Up Stairs on every level at the same position as the Down Stairs
on the level above


The Formulae
------------

All the numeric relationship between the different factors (state of mind,
tolerance, strength, confidence, courage, attack/defense) are codified here.

They are very likely to evolve based on the gameplay experience so let's just
start something easy to implement.

### State of Mind

Starts at: 20
Minimum: 1
Maximum: 100

### Will

Starts at: 5
Minimum: 1
Maximum: 10

At Will 1, the PC cannot resist a dose 2 blocks away.
At Will 2, the PC cannot resist a dose 1 block away.
At Will 10, the PC has 2 action points.

### Tolerance

Starts at 0, increases by one with every Dose used

### Using a Dose

Note: we should probably increase the initial dose strengths -- to around 60 or
so -- so that the player overdoses immediately if they take two in a quick
succession.

Adds the Dose's bonuses of Strength, Confidence, Nerve and Will. The Strength
bonus is reduced by Tolerance. The other attribute bonuses are not affected
by Tolerance.

Example:

The PC has Tolerance 5, State of Mind 20, and all the other attributes at 5.
They use a Dose(str: 30, con: 5, ner: 2, wil: 2).

The result will be:
Tolerance: 6
State of Mind: 45 (20 + (30 - 5))
Confidence: 10 (5 + 5)
Nerve: 7 (5 + 2)
Will: 7 (5 + 2)

### State of Mind

Decreases by one every turn

### Desperation

Player's State of Mind is at or below 20% of the maximum minus 1% per a Will
point.

### Panic

Player's Nerve is at or below 15% of the maximum minus 1% per a Nerve point.
(this is probably no longer necessary when we have enemies that stun and give
panic with a single hit)

### Score

Keep count of used doses.

Keep count of doses that took it close (within 5 points of the limit).

Keep count of killed enemies.

Keep count of the percentage of explored area before leaving.

Whenever we move to a new location, we sum the compound score from the previous
location and multiply it with the percentage of the explored area (10% explored
means getting 10% of the score.). The rationale being that it's harder to stay
on one map and explore it all.



The Combat
----------

Single hitpoint system: everything that gets hit for damage dies.

Every hit counts. You can't avoid the effect once you're hit.

Not all monsters deal damage, though. Some may "merely" reduce your State of
Mind. Or drain your attributes. Or stun you for a few turns. Or frighten you for
a few turns (you lose control and run around randomly).

This means we'll have to rework the PC's attributes, too: Nerve and Will and
Confidence now need to have different meanings and effects in the game.


### Adverse effects ###

* Death
* Attribute drain (state of mind, will, courage)
* Stun (the player can't move for a few turns. The monster must disappear,
  obviously -- otherwise we have an infinite loop)
* Panic  -- the player moves around randomly
* Confusion -- inverts controls (left/right, up/down), usually lasts longer
  (because the player can adapt)

The Bestiary
------------

Describe the monsters in the game: their names, behaviour peculiarities, their
attacks, etc.

There doesn't have to be a lot of different kinds. They should be diverse
though. In terms of attributes, behaviour, how they damage the player.


Ideas for names:

* Doubt
* Anxiety
* Hunger
* Regret
* Shame
* Sorrow
* Shadows
* Voices


### Depression

Glyph: `D`
Hit effect: Death
Has two action points.

### Hunger

Glyph: `h`
Hit effect: State of Mind drain
Move in packs like wild wolfs. One can summon the others in the vicinity and
they surround the player.

### Anxiety

Glyph: `a`
Hit effect: Drains 1 Will point.

### Voices
Glyph: `v`
Hit effect: Stun and disappear

### Shadows
Glyph: `S`
Hit effect: Panic and disappear



Additional ideas and maybes
---------------------------

More withdrawal effects:

"Walls closing in" -- when High, everything is roomy and the corridors are wide.
On Withdrawal, rooms get smaller and corridors narrower.

The monster count, strength, aggressivity depending on State of Mind



The Doses have a few easily recognisable sizes (small, normal, large, XL), but
their purity would be randomized so the player never knows how much they're
getting. They could risk overdose at any point.



Inventory: the ability to pick up doses and use them later. Seems to complicate
things a lot but maybe we could make it work?

The complications: we would want at some point the PC to automatically consume
the Doses they find. What if they have some in the inventory? And should it be
possible to hoard them at all? Kind of feels strange -- especially when that
would be teh only pickable item in the game.


Exploration: we should make the surroundings interesting enough to explore. The
the intresting things could appeare more when High and disappear on Low (and the
monsters doing the opposite). The idea is, there should be something the player
*wants* to do. And exploration would fit nicely if we could figure how to make
it interesting.

Here's an idea: each screen will feature a different area/climate with different
structures and colour schemes:

* Desert
* Watery thing
* Interconnected islands
* Tundra


Some bonus/effect/score/achievement for maintaining the High for a long run
without a withdrawal.



TODO
----
* Add a reliable replay system and turn it on
  This will be great when I trigger a bug during the gameplay but don't know how
  to repeat it yet.
* Show the doses' and monsters' locations when hitting the Perfect High (99 or
  98 points SoM)
* Generate a new location when the player leaves the current map
* Give the player a point of Will when they slay 5 Anxieties
* Add the stronger doses (glyph: `I`)
* Add a hound/pack AI to hunger
* Calculate the score
* Show score on death
* Apart from FoV, have a second slightly larger circle that marks areas as
  explored but not immediately visible.
* Add damage animation (when a character is damaged but survives)
  - particle effects seem like a good way to do this
  - not sure if tcod can do this. Worst case we can "shell out" to SDL
* Add death animation (maybe just a fadeout)
* For multi-turn movenents, display the character's path (in faded-out colours)
* The colours when High should not just be bright, they should be psychedelic,
  flashing continuously, etc.
* Add game log (maybe)
* Have a debug and a release mode
  - debug would automatically open a debugger on exception
  - release would log the exception and alert the player
  - could be implemented in: `game.run(debug=False)`
* Catch all exceptions, log them to a file on crash, show a message to the user
  with the absolute location of the file, asking them to send the stacktrace.
  - alternatively, we can HTTP POST it to a notification service we own in the
    background
* Items/abilities you can see and pick up only when you're high
* Add save/load game mechanics


* Consider this: Keep track of the elapsed turns in the top-level `game`
  variable, update it by the `end_of_turn system` and pass `dt` (number of
  elapsed turns, typically 0 or 1) to the systems that want it.

  It doesn't make much sense to keep the turn count in each `Turn` component
  since the value should be uniform.

  There is an argument to be made for not storing persistent game state (what we
  want to save) outside of the entity system, so maybe store this with the
  Player or possibly a new Global State entity?

* Improve the system code by doing automatic entity queries and component checks

    @system(Position, MoveDestination, Turn)
    def movement_system(e, pos, dest, turn, ecm):
        pass  # `e` is guaranteed to have Position, MoveDestination and Turn

    ...

    ecm.run_system(movement_system)


* Add an `update_component` function to ECM that takes a component type and a
  dict of keys and update functions:

    e.set(Position(1, 1))
	e.update(Position, x=lambda x: x+1, y=lambda x: x+1)
	# equivalent to: pos = e.get(Position); e.set(Position._replace(x=pos.x+1, y=pos.y+1))

* Full screen
* Proper colours
* Numpad controls
* Various resolutions (should work on a 800x600 screen, too)
* Make accessible for the colour-blind people
* Death animation (for PC and NPCs)
