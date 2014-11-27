use std::rand::Rng;
use std::rand::distributions::{Weighted, WeightedChoice, IndependentSample};

use item;
use level::{Tile, TileKind};
use monster::Kind;
use point::{mod, Point};
use generators::GeneratedWorld;

pub fn generate_map<R: Rng>(rng: &mut R, (w, h): (int, int), player: Point) -> Vec<(Point, Tile)> {
    let mut weights = [
        Weighted{weight: 610, item: TileKind::Empty},
        Weighted{weight: 390, item: TileKind::Tree},
    ];
    let opts = WeightedChoice::new(weights.as_mut_slice());
    let mut result = vec![];
    for x in range(0, w) {
        for y in range(0, h) {
            // Player always starts at an empty space:
            let kind = match (x, y) == player {
                true => TileKind::Empty,
                false => opts.ind_sample(rng),
            };
            result.push(((x, y), Tile::new(kind)));
        }
    }
    result
}

pub fn generate_monsters<R: Rng>(rng: &mut R, map: &[(Point, Tile)], player: Point) -> Vec<(Point, Kind)> {
    // 3% chance a monster gets spawned
    let monster_count = 5;
    let monster_chance  = 30;
    let mut weights = [
        Weighted{weight: 1000 - monster_chance, item: None},
        Weighted{weight: monster_chance / monster_count, item: Some(Kind::Anxiety)},
        Weighted{weight: monster_chance / monster_count, item: Some(Kind::Depression)},
        Weighted{weight: monster_chance / monster_count, item: Some(Kind::Hunger)},
        Weighted{weight: monster_chance / monster_count, item: Some(Kind::Shadows)},
        Weighted{weight: monster_chance / monster_count, item: Some(Kind::Voices)},
    ];
    let opts = WeightedChoice::new(weights.as_mut_slice());
    let mut result = vec![];
    for &(pos, tile) in map.iter() {
        if tile.kind != TileKind::Empty {
            continue
        }
        // Don't spawn any monsters in near vicinity to the player
        if point::tile_distance(pos, player) < 6 {
            continue
        }
        if let Some(monster) = opts.ind_sample(rng) {
            result.push((pos, monster));
        }
    }
    result
}

pub fn generate_items<R: Rng>(rng: &mut R, map: &[(Point, Tile)]) -> Vec<(Point, item::Kind)> {
    use item::Kind::*;
    let mut weights = [
        Weighted{weight: 1000 , item: None},
        Weighted{weight: 7, item: Some(Dose)},
        Weighted{weight: 3, item: Some(StrongDose)},
        Weighted{weight: 5, item: Some(Food)},
    ];
    let opts = WeightedChoice::new(weights.as_mut_slice());
    let mut result = vec![];
    for &(pos, tile) in map.iter() {
        if tile.kind != TileKind::Empty {
            continue
        }
        if let Some(item) = opts.ind_sample(rng) {
            result.push((pos, item));
        }
    }
    result
}


pub fn generate<R: Rng>(rng: &mut R, w: int, h: int, player: Point) -> GeneratedWorld {
    let map = generate_map(rng, (w, h), player);
    let monsters = generate_monsters(rng, map.as_slice(), player);
    let items = generate_items(rng, map.as_slice());
    (map, monsters, items)
}
