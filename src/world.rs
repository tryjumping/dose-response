use std::collections::HashMap;

use level::{self, Cell, Level, Walkability, Tile, TileKind};
use item::Item;
use point::Point;
use monster::Monster;
use generators::GeneratedWorld;

use rand::{IsaacRng, Rng};

struct Chunk {
    pub rng: IsaacRng,
    pub level: Level,
}


pub struct World {
    seed: u32,
    max_size: i32,
    chunks: HashMap<Point, Chunk>,
}

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

        // Sort monsters by their APs, set their IDs to equal their indexes in state.monsters:
        // state.monsters.sort_by(|a, b| b.max_ap.cmp(&a.max_ap));
        // for (index, m) in state.monsters.iter_mut().enumerate() {
        //     // TODO: UGH. Just use an indexed entity store that pops these up.
        //     unsafe {
        //         m.set_id(index);
        //     }
        //     let chunk_pos = chunk_from_world_pos(m.position);
        //     match state.world.entry(chunk_pos) {
        //         Occupied(mut chunk) => chunk.get_mut().level.set_monster(m.position, m.id(), m),
        //         Vacant(_) => unreachable!()  // All monsters should belong to a chunk
        //     }
        // }

        unimplemented!()
    }

    fn chunk_pos_from_world_pos(&self, pos: Point) -> Point {
        unimplemented!()
    }

    fn chunk(&mut self, pos: Point) -> &mut Chunk {
        // TODO: generate the chunk if it doesn't exist
        self.chunks.entry(pos).or_insert_with(
            || unimplemented!())
    }

    fn cell(&mut self, pos: Point) -> &Cell {
        let chunk_pos = self.chunk_pos_from_world_pos(pos);
        let chunk = self.chunk(chunk_pos);
        // NOTE: the positions within a chunk/level start from zero so
        // we need to de-offset them with the chunk position.
        let level_position = chunk_pos - pos;
        chunk.level.cell(level_position)
    }

    fn cell_mut(&mut self, pos: Point) -> &mut Cell {
        let chunk_pos = self.chunk_pos_from_world_pos(pos);
        let chunk = self.chunk(chunk_pos);
        // NOTE: the positions within a chunk/level start from zero so
        // we need to de-offset them with the chunk position.
        let level_position = chunk_pos - pos;
        chunk.level.cell_mut(level_position)
    }

    /// Check whether the given position is within the bounds of the World.
    ///
    /// While the world should be "technically infinite", we well have
    /// some sort of upper limit on the positions it's able to
    /// support.
    pub fn within_bounds(&self, pos: Point) -> bool {
        pos.x < self.max_size &&
            pos.x > -self.max_size &&
            pos.y < self.max_size &&
            pos.y > -self.max_size
    }


    /// Check whether the given position is walkable.
    ///
    /// Points outside of the World are not walkable. The
    /// `walkability` option controls can influence the logic: are
    /// monster treated as blocking or not?
    pub fn walkable(&mut self, pos: Point, walkability: Walkability) -> bool {
        let walkable = match walkability {
            Walkability::WalkthroughMonsters => true,
            Walkability::BlockingMonsters => self.monster_on_pos(pos).is_none(),
        };
        self.within_bounds(pos) &&
            self.cell(pos).tile.kind == TileKind::Empty &&
            walkable
    }

    /// Change the tile on the given position. If the position is not
    /// within bounds, nothing happens.
    pub fn set_tile(&mut self, pos: Point, tile: Tile) {
        if self.within_bounds(pos) {
            self.cell_mut(pos).tile = tile;
        }
    }

    /// Put an item on the tile at the given position. There can be
    /// multiple items on one tile. If the position is not within
    /// bounds, nothing happens.
    pub fn add_item(&mut self, pos: Point, item: Item) {
        if self.within_bounds(pos) {
            self.cell_mut(pos).items.push(item);
        }
    }

    /// Pick up the top `Item` stacked on the tile. If the position is
    /// not withing bounds, nothing happens.
    pub fn pickup_item(&mut self, pos: Point) -> Option<Item> {
        if self.within_bounds(pos) {
            self.cell_mut(pos).items.pop()
        } else {
            None
        }
    }

    pub fn monster_on_pos(&mut self, pos: Point) -> Option<usize> {
        unimplemented!()
    }

    pub fn move_monster(&mut self, monster: &mut Monster, dest: Point) {
        unimplemented!()
    }

    pub fn remove_monster(&mut self, id: usize, monster: &mut Monster) {
        unimplemented!()
    }

    pub fn explore(&mut self, pos: Point, radius: i32) {
        unimplemented!()
    }

    pub fn nearest_dose(&mut self, pos: Point, radius: i32) -> Option<(Point, Item)> {
        unimplemented!()
    }

    pub fn random_neighbour_position<T: Rng>(&mut self, rng: &mut T,
                                             starting_pos: Point,
                                             walkability: Walkability) -> Point
    {
        unimplemented!()
    }

    pub fn iter(&mut self) -> level::Cells {
        unimplemented!()
    }

    pub fn iter_mut(&mut self) -> level::CellsMut {
        unimplemented!()
    }
}


pub fn populate_world(world: &mut World,
                      monsters: &mut Vec<Monster>,
                      generated_world: GeneratedWorld) {
    let (map, generated_monsters, items) = generated_world;
    for &(pos, item) in map.iter() {
        world.set_tile(pos, item);
    }
    for &(pos, kind) in generated_monsters.iter() {
        assert!(world.walkable(pos, Walkability::BlockingMonsters));
        let monster = Monster::new(kind, pos);
        monsters.push(monster);
    }
    for &(pos, item) in items.iter() {
        assert!(world.walkable(pos, Walkability::BlockingMonsters));
        world.add_item(pos, item);
    }
}

pub fn random_neighbour_position<R: Rng>(rng: R, pos: Point, walkability: Walkability) -> Point {
    unimplemented!()
}

pub fn nearest_dose(pos: Point, radius: i32) -> Option<(Point, Item)> {
    unimplemented!()
}
