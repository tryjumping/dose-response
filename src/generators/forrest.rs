use std::rand::Rng;
use std::rand::distributions::{Weighted, WeightedChoice, IndependentSample};

use item::{mod, Item};
use level::{Tile, TileKind};
use monster::Kind;
use player::Modifier;
use point::{mod, Point};
use generators::GeneratedWorld;

fn generate_map<R: Rng>(rng: &mut R, (w, h): (int, int), player: Point) -> Vec<(Point, Tile)> {
    let mut weights = [
        Weighted{weight: 610, item: TileKind::Empty},
        Weighted{weight: 390, item: TileKind::Tree},
    ];
    let opts = WeightedChoice::new(weights.as_mut_slice());
    let mut result = vec![];
    // NOTE: starting with `y` seems weird but it'll generate the right pattern:
    // start at top left corner, moving to the right
    for y in range(0, h) {
        for x in range(0, w) {
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

fn generate_monsters<R: Rng>(rng: &mut R, map: &[(Point, Tile)], player: Point) -> Vec<(Point, Kind)> {
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

fn new_item<R: Rng>(kind: item::Kind, rng: &mut R) -> Item {
    use item::Kind::*;
    let modifier = match kind {
        Dose => Modifier::Intoxication{state_of_mind: 72 + rng.gen_range(-5, 6),
                                       tolerance_increase: 1},
        StrongDose => Modifier::Intoxication{state_of_mind: 130 + rng.gen_range(-15, 16),
                                             tolerance_increase: 2},
        Food => Modifier::Attribute{state_of_mind: 10,
                                    will: 0},
    };
    Item {
        kind: kind,
        modifier: modifier,
    }
}

fn generate_items<R: Rng>(rng: &mut R, map: &[(Point, Tile)], (px, py): Point) -> Vec<(Point, item::Item)> {
    use item::Kind::*;
    let pos_offset = &[-4, -3, -2, -1, 1, 2, 3, 4];
    let mut initial_dose = (px + *rng.choose(pos_offset).unwrap(),
                            py + *rng.choose(pos_offset).unwrap());
    let mut weights_near_player = [
        Weighted{weight: 1000 , item: None},
        Weighted{weight: 2, item: Some(Dose)},
        Weighted{weight: 0, item: Some(StrongDose)},
        Weighted{weight: 20, item: Some(Food)},

    ];
    let mut weights_rest = [
        Weighted{weight: 1000 , item: None},
        Weighted{weight: 7, item: Some(Dose)},
        Weighted{weight: 3, item: Some(StrongDose)},
        Weighted{weight: 5, item: Some(Food)},
    ];
    let gen_near_player = WeightedChoice::new(weights_near_player.as_mut_slice());
    let gen_rest = WeightedChoice::new(weights_rest.as_mut_slice());

    let mut result = vec![];
    for &(pos, tile) in map.iter() {
        match tile.kind {
            // Initial dose position is blocked, move it somewhere else:
            TileKind::Tree if pos == initial_dose => {
                initial_dose = (initial_dose.0 + 1, initial_dose.1);
                if point::tile_distance(initial_dose, (px, py)) > 4 {
                    initial_dose = (initial_dose.0 - 4, initial_dose.1 + 1);
                }
            }
            TileKind::Tree => {
                // Occupied tile, do nothing:
            }
            TileKind::Empty if pos == initial_dose => {
                result.push((pos, new_item(Dose, rng)));
            }
            TileKind::Empty => {
                let gen = match point::tile_distance(pos, (px, py)) < 6 {
                    true => &gen_near_player,
                    false => &gen_rest,
                };
                if let Some(kind) = gen.ind_sample(rng) {
                    result.push((pos, new_item(kind, rng)));
                }
            }
        }
    }
    result
}


pub fn generate<R: Rng>(rng: &mut R, w: int, h: int, player: Point) -> GeneratedWorld {
    let map = generate_map(rng, (w, h), player);
    let monsters = generate_monsters(rng, map.as_slice(), player);
    let items = generate_items(rng, map.as_slice(), player);
    (map, monsters, items)
}
