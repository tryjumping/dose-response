use rand::Rng;
use rand::distributions::{Weighted, WeightedChoice, IndependentSample};

use item::{self, Item};
use level::{Tile, TileKind};
use monster::Kind;
use player::Modifier;
use point::Point;
use generators::GeneratedWorld;

fn generate_map<R: Rng>(rng: &mut R, map_size: Point, player_pos: Point) -> Vec<(Point, Tile)> {
    let mut weights = [
        Weighted{weight: 610, item: TileKind::Empty},
        Weighted{weight: 390, item: TileKind::Tree},
    ];
    let opts = WeightedChoice::new(&mut weights);
    let mut result = vec![];
    // NOTE: starting with `y` seems weird but it'll generate the right pattern:
    // start at top left corner, moving to the right
    for y in 0..map_size.y {
        for x in 0..map_size.x {
            // Player always starts at an empty space:
            let kind = match player_pos == (x, y) {
                true => TileKind::Empty,
                false => opts.ind_sample(rng),
            };
            result.push((Point::new(x, y), Tile::new(kind)));
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
    let opts = WeightedChoice::new(&mut weights);
    let mut result = vec![];
    for &(pos, tile) in map.iter() {
        if tile.kind != TileKind::Empty {
            continue
        }
        // Don't spawn any monsters in near vicinity to the player
        if player.tile_distance(pos) < 6 {
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
    let mut irresistible = 0;
    let modifier = match kind {
        Dose => {
            let mut dose_w = [
                Weighted{weight: 7, item: 72},
                Weighted{weight: 3, item: 130}
            ];
            let base_strength_gen = WeightedChoice::new(&mut dose_w);
            let base = base_strength_gen.ind_sample(rng);
            let (strength, tolerance, r) = match base <= 100 {
                true => (base + rng.gen_range(-5, 6), 1, 2),
                false => (base + rng.gen_range(-15, 16), 2, 3),
            };
            irresistible = r;
            Modifier::Intoxication{state_of_mind: strength,
                                   tolerance_increase: tolerance}
        },
        Food => Modifier::Attribute{state_of_mind: 10,
                                    will: 0},
    };
    Item {
        kind: kind,
        modifier: modifier,
        irresistible: irresistible,
    }
}


fn generate_items<R: Rng>(rng: &mut R, map: &[(Point, Tile)], player_pos: Point) -> Vec<(Point, item::Item)> {
    use item::Kind::*;
    let pos_offset = &[-4, -3, -2, -1, 1, 2, 3, 4];

    let mut initial_dose = player_pos + Point::new(*rng.choose(pos_offset).unwrap(),
                                                   *rng.choose(pos_offset).unwrap());
    let mut weights_near_player = [
        Weighted{weight: 1000 , item: None},
        Weighted{weight: 2, item: Some(Dose)},
        Weighted{weight: 20, item: Some(Food)},
    ];

    let mut weights_rest = [
        Weighted{weight: 1000 , item: None},
        Weighted{weight: 10, item: Some(Dose)},
        Weighted{weight: 5, item: Some(Food)},
    ];

    let gen_near_player = WeightedChoice::new(&mut weights_near_player);
    let gen_rest = WeightedChoice::new(&mut weights_rest);

    let mut result = vec![];
    for &(pos, tile) in map.iter() {
        // Don't create an item where the player starts:
        if pos == player_pos {
            continue
        }
        match tile.kind {
            // Initial dose position is blocked, move it somewhere else:
            TileKind::Tree if pos == initial_dose => {
                initial_dose += (1, 0);
                if player_pos.tile_distance(initial_dose) > 4 {
                    initial_dose += (-4, 1);
                }
            }
            TileKind::Tree => {
                // Occupied tile, do nothing.
            }
            TileKind::Empty if pos == initial_dose => {
                result.push((pos, new_item(Dose, rng)));
            }
            TileKind::Empty => {
                let gen = match player_pos.tile_distance(pos) < 6 {
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


pub fn generate<R: Rng>(rng: &mut R, size: Point, player: Point) -> GeneratedWorld {
    let map = generate_map(rng, size, player);
    let monsters = generate_monsters(rng, &map, player);
    let items = generate_items(rng, &map, player);
    (map, monsters, items)
}
