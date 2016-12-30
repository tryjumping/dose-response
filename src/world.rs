use std::collections::HashMap;

use level::{Cell, Level, Walkability, TileKind};
use item::{self, Item};
use point::{Point, CircularArea};
use monster::Monster;
use generators::{self, GeneratedWorld};

use rand::{IsaacRng, Rng, SeedableRng};

struct Chunk {
    position: Point,
    pub rng: IsaacRng,
    pub level: Level,
    monsters: Vec<Monster>,
}

impl Chunk {
    fn new(world_seed: u32, position: ChunkPosition, size: i32, player_position: Point) -> Self {
        let pos = position.position;
        // NOTE: `x` and `y` overflow on negative values here, but all
        // we care about is having a distinct value for each position
        // so our seeds don't repeat. So this is fine here.
        let chunk_seed: &[_] = &[world_seed, pos.x as u32, pos.y as u32];

        // TODO: Monsters in different chunks will now have identical
        // IDs. We need to investigate whether that's a problem.

        let mut chunk = Chunk {
            position: pos,
            rng: SeedableRng::from_seed(chunk_seed),
            level: Level::new(size, size),
            monsters: vec![],
        };

        let generated_data = generators::forrest::generate(&mut chunk.rng, chunk.level.size(), player_position);

        chunk.populate(generated_data);

        // TODO: set the monsters to the level and fix their IDs here.

        chunk
    }


    fn populate(&mut self, generated_world: GeneratedWorld) {
        let (map, generated_monsters, items) = generated_world;
        for &(pos, item) in map.iter() {
            self.level.set_tile(pos, item);
        }
        for &(pos, kind) in generated_monsters.iter() {
            assert!(self.level.walkable(pos, Walkability::BlockingMonsters));
            let monster = Monster::new(kind, pos);
            self.monsters.push(monster);
        }
        for &(pos, item) in items.iter() {
            assert!(self.level.walkable(pos, Walkability::BlockingMonsters));
            self.level.add_item(pos, item);
        }
    }

}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
struct ChunkPosition {
    position: Point,
}


pub struct World {
    seed: u32,
    max_half_size: i32,
    chunk_size: i32,
    chunks: HashMap<ChunkPosition, Chunk>,
}

impl World {
    pub fn new(seed: u32, dimension: i32, chunk_size: i32) -> Self {
        assert!(dimension > 0);
        assert!(chunk_size > 0);
        assert_eq!(dimension % 2, 0);
        assert_eq!(dimension % chunk_size, 0);
        World {
            seed: seed,
            max_half_size: dimension / 2,
            chunk_size: chunk_size,
            chunks: HashMap::new(),
        }
    }

    /// Return the ChunkPosition for a given point within the chunk.
    ///
    /// Chunks have equal width and height and can have negative
    /// positions. There is a chunk at `(0, 0)` and then at
    /// `(-chunk_size, 0)`, `(chunk_size, 0)` and so on.
    fn chunk_pos_from_world_pos(&self, pos: Point) -> ChunkPosition {
        let chunk_pos = |num: i32| {
            let size = self.chunk_size;
            if num >= 0 {
                (num / size) * size
            } else {
                (-(((-num - 1) / size) + 1)) * size
            }
        };

        ChunkPosition {
            position: Point {
                x: chunk_pos(pos.x),
                y: chunk_pos(pos.y),
            }
        }
    }

    /// Get the chunk at the given world position. This means it
    /// doesn't have to match chunk's internal position -- any point
    /// within that Chunk will do.
    fn chunk(&mut self, pos: Point) -> &mut Chunk {
        let chunk_position = self.chunk_pos_from_world_pos(pos);

        let seed = self.seed;
        let chunk_size = self.chunk_size;
        // TODO: figure out how to generate the starting chunks so the
        // player has some doses and food and no monsters.
        self.chunks.entry(chunk_position).or_insert_with(
            || Chunk::new(seed, chunk_position, chunk_size, (0, 0).into()))
    }

    fn cell(&mut self, pos: Point) -> &Cell {
        let chunk = self.chunk(pos);
        // NOTE: the positions within a chunk/level start from zero so
        // we need to de-offset them with the chunk position.
        let level_position = pos - chunk.position;
        chunk.level.cell(level_position)
    }

    fn cell_mut(&mut self, pos: Point) -> &mut Cell {
        let chunk = self.chunk(pos);
        // NOTE: the positions within a chunk/level start from zero so
        // we need to de-offset them with the chunk position.
        let level_position = pos - chunk.position;
        chunk.level.cell_mut(level_position)
    }

    /// Check whether the given position is within the bounds of the World.
    ///
    /// While the world should be "technically infinite", we well have
    /// some sort of upper limit on the positions it's able to
    /// support.
    pub fn within_bounds(&self, pos: Point) -> bool {
        pos.x < self.max_half_size &&
            pos.x > -self.max_half_size &&
            pos.y < self.max_half_size &&
            pos.y > -self.max_half_size
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

    /// Pick up the top `Item` stacked on the tile. If the position is
    /// not withing bounds, nothing happens.
    pub fn pickup_item(&mut self, pos: Point) -> Option<Item> {
        if self.within_bounds(pos) {
            self.cell_mut(pos).items.pop()
        } else {
            None
        }
    }

    /// If there's a monster at the given tile, return its ID.
    ///
    /// Returns `None` if there is no monster or if `pos` is out of bounds.
    pub fn monster_on_pos(&mut self, pos: Point) -> Option<usize> {
        if self.within_bounds(pos) {
            self.chunk(pos).level.monster_on_pos(pos)
        } else {
            None
        }
    }

    pub fn move_monster(&mut self, monster: &mut Monster, dest: Point) {
        let monster_chunk_pos = self.chunk_pos_from_world_pos(monster.position);
        let destination_chunk_pos = self.chunk_pos_from_world_pos(dest);
        if monster_chunk_pos == destination_chunk_pos {
            self.chunk(monster.position).level.move_monster(monster, dest);
        } else {
            // TODO: the monster needs to move to a different chunk!

            // TODO: we'll need to remove the monster from the
            // chunk.monsters list which could interfere with the
            // indices, which could be a problem.
            //
            // Plus, taking a pointer to the monster will cause a
            // problem here -- since the monster in the destination
            // chunk will literally be a different monster. So either
            // we do maintain the global list of all monsters, or have
            // to rework this logic.
            unimplemented!()
        }
    }

    /// Remove the given monster from the world.
    pub fn remove_monster(&mut self, id: usize, monster: &mut Monster) {
        // TODO: This is problematic -- we shouldn't be passing a
        // mutable pointer here. Ideally, the monster should no longer
        // be available if removed or we should return it.
        self.chunk(monster.position).level.remove_monster(id, monster)
    }

    /// Set cells within the given radius as explored.
    pub fn explore(&mut self, centre: Point, radius: i32) {
        for pos in CircularArea::new(centre, radius) {
            if self.within_bounds(pos) {
                self.cell_mut(pos).explored = true;
            }
        }
    }

    /// Get a dose within the given radius that's nearest to the specified point.
    pub fn nearest_dose(&mut self, centre: Point, radius: i32) -> Option<(Point, Item)> {
        let mut doses = vec![];
        for pos in CircularArea::new(centre, radius) {
            // Make sure we don't go out of bounds with self.cell(pos):
            if !self.walkable(pos, Walkability::WalkthroughMonsters) {
                continue
            }
            for &item in self.cell(pos).items.iter() {
                match item.kind {
                    item::Kind::Dose | item::Kind::StrongDose => {
                        doses.push((pos, item));
                    }
                    item::Kind::Food => {},
                }
            }
        }

        doses.pop().map(|dose| {
            let mut result = dose;
            for d in &doses {
                if centre.tile_distance(d.0) < centre.tile_distance(result.0) {
                    result = *d;
                }
            }
            result
        })
    }

    /// Return a random walkable position next to the given point.
    ///
    /// If there is no such position available, return `starting_pos`.
    pub fn random_neighbour_position<T: Rng>(&mut self, rng: &mut T,
                                             starting_pos: Point,
                                             walkability: Walkability) -> Point
    {
        let mut walkables = vec![];
        for pos in starting_pos.square_area(1) {
            if pos != starting_pos && self.walkable(pos, walkability) {
                walkables.push(pos)
            }
        }
        match rng.choose(&walkables) {
            Some(&random_pos) => random_pos,
            None => starting_pos  // Nowhere to go
        }
    }

    /// Return an iterator over `Cell` that covers a rectangular shape
    /// specified by the top-left (inclusive) point and the dimensions
    /// (width, height) of the rectangle.
    ///
    /// The iteration order is not specified.
    pub fn with_cells<F>(&mut self, top_left: Point, dimensions: Point, mut callback: F)
        where F: FnMut(Point, &Cell)
    {
        assert!(dimensions.x >= 0);
        assert!(dimensions.y >= 0);
        let bottom_right = top_left + dimensions;

        let chunk_size = self.chunk_size;
        let mut chunk_pos = self.chunk_pos_from_world_pos(top_left).position;
        let starter_chunk_x = chunk_pos.x;

        while chunk_pos.y < bottom_right.y {
            while chunk_pos.x < bottom_right.x {
                let chunk = self.chunk(chunk_pos);
                for (cell_level_pos, cell) in chunk.level.iter() {
                    let cell_world_pos = cell_level_pos + chunk_pos;
                    if cell_world_pos >= top_left && cell_world_pos <= bottom_right {
                        callback(cell_world_pos, cell);
                    }
                }
                chunk_pos.x += chunk_size;
            }
            chunk_pos.y += chunk_size;
            chunk_pos.x = starter_chunk_x;
        }
    }

    pub fn iter_mut(&mut self) -> CellsMut {
        // NOTE: this is used to update the animation of each cell
        // NOTE: it's now a no-op because we need to rethink this approach somewhat.
        CellsMut {
            phantom: ::std::marker::PhantomData
        }
    }

}

pub struct CellsMut<'a> {
    phantom: ::std::marker::PhantomData<&'a mut Cell>,
}


impl<'a> Iterator for CellsMut<'a> {
    type Item = (Point, &'a mut Cell);

    fn next(&mut self) -> Option<(Point, &'a mut Cell)> {
        None
    }
}
