Data in the game
----------------

1. Position of the Player Character (including which floor they're on)
   * position.x
   * position.y
   * position.floor
2. Position of each Empty Tile
   * position.x
   * position.y
   * position.floor
3. Position of each Wall Tile
   * position.x
   * position.y
   * position.floor
4. Position of each Door Tile
   * position.x
   * position.y
   * position.floor
5. Open state of each Door (open/closed)
   * isSolid
   * isOpenable
6. Position of each Stairs
   * position.x
   * position.y
   * position.floor
7. Destination of each Stairs (relative)
   * destination.dx
   * destination.dy
   * destination.df
8. Current value of each PC's Attribute (Confidence, Nerve, Will)
   * attribute.confidence
   * attribute.nerve
   * attribute.will
9. Number of turns that elapsed
   * statistics.turns
10. Position of each Dose
   * position.x
   * position.y
   * position.floor
11. Strength and attribute bonuses for each Dose
   * attribute-modifier.state-of-mind
   * attribute-modifier.confidence ... nerve ... will ... tolerance
12. Current Tolerance value of PC
   * attribute.tolerance
13. Current State of Mind value of PC
   * attribute.state-of-mind
14. Graphics (sprite/glyph) for each rendered entity
   * tile.type (enum)
   * tile.color
   * tile.glyph
15. Monster Strength
   * monster.strength
16. Position of each Monster
   * position.x
   * position.y
   * position.floor
17. Monster victory logic
   * attribute-modifier.state-of-mind
   * attribute-modifier.confidence ... nerve ... will ... tolerance
   * effect-multiplier.damage (float)
18. Monster AI state (idle/attacking)
   * ai.state
19. Monster type
   * monster.type
20. Monster name
   * info.name
   * info.description
21. Player name
   * info.name
21. PC Desperation status
   * adverse-effects.desperation (bool)
22. PC Panic status
   * adverse-effects.panic (bool)


Components
----------

1. Position [x y floor]
2. Solid
   * flag
3. Openable [state]
   * state: enum (open/closed)
4. Destination [dx dy df]
   * all ints
   * relative positions of the destination
5. Attribute [state-of-mind tolerance confidence nerve will]
6. Statistics [turns kills doses]
7. AttributeModifier [state-of-mind tolerance confidence nerve will]
8. Tile [type color glyph]
   * type: enum (empty, wall, player, dose, door, stairs, anxiety, hunger)
9. Monster [kind strength]
10. AI [state]
    * state: enum (idle/attacking)
11. Info [name description]
12. CurrentEffects [desperation panic]
    * bools
13. EffectMultiplier [damage]
    * damage: float
14. Interactive
    - for stairs, doses, maybe even doors?
    - possible interactions:
      * teleportation (Destination component)
      * change attributes (AttributeModifier component)
      * open/close (Openable component)
