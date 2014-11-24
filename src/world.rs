use std::rand::Rng;

use level::{Level, Tile};
use monster::Monster;
use generators::GeneratedWorld;
use point;


pub fn populate_world<R: Rng>(world_size: (int, int),
                              level: &mut Level,
                              monsters: &mut Vec<Monster>,
                              player_pos: (int, int),
                              rng: &mut R,
                              generate: fn(&mut R, int, int) -> GeneratedWorld) {
    // TODO: this closure doesn't seem to work correctly (set all tiles to e.g.
    // monsters and look at the shape of the gap this produces):
    let near_player = |x, y| point::tile_distance(player_pos, (x, y)) < 6;
    let pos_offset = &[-4, -3, -2, -1, 1, 2, 3, 4];
    let initial_dose_pos = (player_pos.0 + *rng.choose(pos_offset).unwrap(),
                            player_pos.1 + *rng.choose(pos_offset).unwrap());
    let mut initial_foods_pos = Vec::<(int, int)>::new();
    // TODO: move this random generation stuff to generators?
    for _ in range(0, rng.gen_range::<uint>(1, 4)) {
        let pos = (player_pos.0 + *rng.choose(pos_offset).unwrap(),
                   player_pos.1 + *rng.choose(pos_offset).unwrap());
        initial_foods_pos.push(pos);
    };
    let (width, height) = world_size;
    let (map, generated_monsters, items) = generate(rng, width, height);
    for &(pos, item) in map.iter() {
        let tile = if pos == player_pos {
            // Player should always start on an empty tile:
            Tile::Empty
        } else {
            item
        };
        level.set_tile(pos, tile);
    }
    for &(pos, kind) in generated_monsters.iter() {
        assert!(level.walkable(pos));
        let monster = Monster::new(kind, pos);
        monsters.push(monster);
    }
    for &(pos, item) in items.iter() {
        assert!(level.walkable(pos));
        level.add_item(pos, item);
    }
}
