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


pub struct World;

impl World {
    pub fn new() -> Self {
        unimplemented!()
    }

    fn generate_chunk(&mut self, pos: Point) {
        // let map_dimensions: Point = (state.map_size, state.map_size).into();
        // let left_top_corner = state.screen_position_in_world - map_dimensions / 2;
        // // NOTE: The world goes from (0, 0) onwards. So `x / chunk_size`
        // // gives you the horizontal coordinate of the chunk containing
        // // your `x`.
        // let min_x_chunk = left_top_corner.x / state.chunk_size;
        // let x_cells_to_fill = left_top_corner.x - min_x_chunk + state.map_size;
        // let x_chunks = if x_cells_to_fill % state.chunk_size == 0 {
        //     x_cells_to_fill / state.chunk_size
        // } else {
        //     x_cells_to_fill / state.chunk_size + 1
        // };

        // let min_y_chunk = left_top_corner.y / state.chunk_size;
        // let y_cells_to_fill = left_top_corner.y - min_y_chunk + state.map_size;
        // let y_chunks = if y_cells_to_fill % state.chunk_size == 0 {
        //     y_cells_to_fill / state.chunk_size
        // } else {
        //     y_cells_to_fill / state.chunk_size + 1
        // };

        // let min_chunk_pos = Point::new(min_x_chunk, min_y_chunk);

        // for x_chunk_increment in 0..x_chunks {
        //     for y_chunk_increment in 0..y_chunks {
        //         let chunk_pos = min_chunk_pos + (x_chunk_increment, y_chunk_increment);
        //         assert!(chunk_pos.x >= 0);
        //         assert!(chunk_pos.y >= 0);

        //         let chunk_seed: &[_] = &[state.seed, chunk_pos.x as u32, chunk_pos.y as u32];
        //         let mut chunk = Chunk {
        //             rng: SeedableRng::from_seed(chunk_seed),
        //             level: Level::new(state.chunk_size, state.chunk_size),
        //         };

        //         let generated_level = generators::forrest::generate(&mut chunk.rng,
        //                                                             chunk.level.size(),
        //                                                             state.player.pos);
        //         world::populate_world(&mut chunk.level,
        //                               &mut state.monsters,
        //                               generated_level);

        //         state.world.insert(chunk_pos, chunk);
        //     }
        // }
        unimplemented!()
    }
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
