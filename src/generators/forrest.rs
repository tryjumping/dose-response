use rand::Rng;
use rand::distributions::{Weighted, WeightedChoice, IndependentSample};

use item::{self, Item};
use level::{Tile, TileKind};
use monster::Kind;
use player::Modifier;
use point::Point;
use generators::GeneratedWorld;

fn generate_map<R: Rng>(rng: &mut R,
                        map_size: Point,
                        player_pos: Point)
                        -> Vec<(Point, Tile)> {
    let mut weights = [Weighted {
                           weight: 610,
                           item: TileKind::Empty,
                       },
                       Weighted {
                           weight: 390,
                           item: TileKind::Tree,
                       }];
    let opts = WeightedChoice::new(&mut weights);
    let mut result = vec![];
    // NOTE: starting with `y` seems weird but it'll generate the right pattern:
    // start at top left corner, moving to the right
    for y in 0..map_size.y {
        for x in 0..map_size.x {
            // TODO: due to coordinate conversion, this is wrong for
            // every chunk but the one the player is in.
            //
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

fn generate_monsters<R: Rng>(rng: &mut R,
                             map: &[(Point, Tile)])
                             -> Vec<(Point, Kind)> {
    // 3% chance a monster gets spawned
    let monster_count = 5;
    let monster_chance = 30;
    let mut weights = [Weighted {
                           weight: 1000 - monster_chance,
                           item: None,
                       },
                       Weighted {
                           weight: monster_chance / monster_count,
                           item: Some(Kind::Anxiety),
                       },
                       Weighted {
                           weight: monster_chance / monster_count,
                           item: Some(Kind::Depression),
                       },
                       Weighted {
                           weight: monster_chance / monster_count,
                           item: Some(Kind::Hunger),
                       },
                       Weighted {
                           weight: monster_chance / monster_count,
                           item: Some(Kind::Shadows),
                       },
                       Weighted {
                           weight: monster_chance / monster_count,
                           item: Some(Kind::Voices),
                       },
                       Weighted {
                           weight: 1,
                           item: Some(Kind::Npc),
                       }];
    let opts = WeightedChoice::new(&mut weights);
    let mut result = vec![];
    for &(pos, tile) in map.iter() {
        if tile.kind != TileKind::Empty {
            continue;
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
            irresistible = 2;
            let base = 70;
            Modifier::Intoxication {
                state_of_mind: base + rng.gen_range(-5, 6),
                tolerance_increase: 1,
            }
        }
        StrongDose => {
            irresistible = 4;
            let base = 130;
            Modifier::Intoxication {
                state_of_mind: base + rng.gen_range(-15, 16),
                tolerance_increase: 3,
            }
        }
        CardinalDose => {
            irresistible = 3;
            let base = 95;
            Modifier::Intoxication {
                state_of_mind: base + rng.gen_range(-10, 11),
                tolerance_increase: 2,
            }
        }
        DiagonalDose => {
            irresistible = 3;
            let base = 95;
            Modifier::Intoxication {
                state_of_mind: base + rng.gen_range(-10, 11),
                tolerance_increase: 2,
            }
        }
        Food => Modifier::Attribute { state_of_mind: 10, will: 0 },
    };
    Item {
        kind: kind,
        modifier: modifier,
        irresistible: irresistible,
    }
}


fn generate_items<R: Rng>(rng: &mut R,
                          map: &[(Point, Tile)])
                          -> Vec<(Point, item::Item)> {
    use item::Kind::*;

    let mut weights = [Weighted { weight: 1000, item: None },
                       Weighted { weight: 8, item: Some(Dose) },
                       Weighted {
                           weight: 3,
                           item: Some(StrongDose),
                       },
                       Weighted {
                           weight: 2,
                           item: Some(CardinalDose),
                       },
                       Weighted {
                           weight: 2,
                           item: Some(DiagonalDose),
                       },
                       Weighted { weight: 5, item: Some(Food) }];

    let generator = WeightedChoice::new(&mut weights);

    let mut result = vec![];
    for &(pos, tile) in map.iter() {
        match tile.kind {
            TileKind::Tree => {
                // Occupied tile, do nothing.
            }
            TileKind::Empty => {
                if let Some(kind) = generator.ind_sample(rng) {
                    result.push((pos, new_item(kind, rng)));
                }
            }
        }
    }
    result
}


pub fn generate<R: Rng>(rng: &mut R,
                        size: Point,
                        player: Point)
                        -> GeneratedWorld {
    let map = generate_map(rng, size, player);
    let monsters = generate_monsters(rng, &map);
    let items = generate_items(rng, &map);
    (map, monsters, items)
}
