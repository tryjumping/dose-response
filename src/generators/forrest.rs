use std::rand::Rng;
use std::rand::distributions::{Weighted, WeightedChoice, IndependentSample};

use item;
use level::{Tile, TileKind};
use monster::Kind;
use point::Point;
use generators::GeneratedWorld;

pub fn generate_map<R: Rng>(rng: &mut R, w: int, h: int) -> Vec<(Point, Tile)> {
    let mut weights = [
        Weighted{weight: 610, item: TileKind::Empty},
        Weighted{weight: 390, item: TileKind::Tree},
    ];
    let opts = WeightedChoice::new(weights.as_mut_slice());
    let mut result = vec![];
    for x in range(0, w) {
        for y in range(0, h) {
            result.push(((x, y), Tile::new(opts.ind_sample(rng))));
        }
    }
    result
}

pub fn generate_monsters<R: Rng>(rng: &mut R, map: &[(Point, Tile)]) -> Vec<(Point, Kind)> {
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


pub fn generate<R: Rng>(rng: &mut R, w: int, h: int) -> GeneratedWorld {
    let map = generate_map(rng, w, h);
    let monsters = generate_monsters(rng, map.as_slice());
    let items = generate_items(rng, map.as_slice());
    (map, monsters, items)
}
