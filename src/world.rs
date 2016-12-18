use level::{Level, Walkability};
use item::Item;
use point::Point;
use monster::Monster;
use generators::GeneratedWorld;

use rand::{IsaacRng, Rng};

pub struct Chunk {
    pub rng: IsaacRng,
    pub level: Level,
}


pub fn populate_world(level: &mut Level,
                      monsters: &mut Vec<Monster>,
                      generated_world: GeneratedWorld) {
    let (map, generated_monsters, items) = generated_world;
    for &(pos, item) in map.iter() {
        level.set_tile(pos, item);
    }
    for &(pos, kind) in generated_monsters.iter() {
        assert!(level.walkable(pos, Walkability::BlockingMonsters));
        let monster = Monster::new(kind, pos);
        monsters.push(monster);
    }
    for &(pos, item) in items.iter() {
        assert!(level.walkable(pos, Walkability::BlockingMonsters));
        level.add_item(pos, item);
    }
}

pub fn random_neighbour_position<R: Rng>(rng: R, pos: Point, walkability: Walkability) -> Point {
    unimplemented!()
}

pub fn nearest_dose(pos: Point, radius: i32) -> Option<(Point, Item)> {
    unimplemented!()
}
