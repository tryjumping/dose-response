use crate::generators::GeneratedWorld;

use crate::color;
use crate::formula;
use crate::item::{self, Item};
use crate::level::{Tile, TileKind};
use crate::monster::{Kind, Monster};
use crate::player::Modifier;
use crate::point::Point;

use rand::{seq::SliceRandom, Rng};

// TODO: Instead of `map_size`, use a Rectangle with the world
// positions here. We want to expose the non-world coordinates in as
// few places as possible.
fn generate_map<R: Rng, G: Rng>(
    rng: &mut R,
    throwavay_rng: &mut G,
    map_size: Point,
    player_pos: Point,
) -> Vec<(Point, Tile)> {
    assert!(formula::CHUNK_DENSITY_VARIABILITY.0 < formula::CHUNK_DENSITY_VARIABILITY.1);
    assert!(formula::CHUNK_BASELINE_DENSITY + formula::CHUNK_DENSITY_VARIABILITY.0 > 0.0);
    assert!(formula::CHUNK_BASELINE_DENSITY + formula::CHUNK_DENSITY_VARIABILITY.1 < 1.0);

    let density = formula::CHUNK_BASELINE_DENSITY
        + rng.gen_range(
            formula::CHUNK_DENSITY_VARIABILITY.0,
            formula::CHUNK_DENSITY_VARIABILITY.1,
        );
    let occupied_count = (density * 100.0) as i32;
    let choices = [
        (TileKind::Empty, 100 - occupied_count),
        (TileKind::Tree, occupied_count),
    ];
    let mut result = vec![];
    // NOTE: starting with `y` seems weird but it'll generate the right pattern:
    // start at top left corner, moving to the right
    for y in 0..map_size.y {
        for x in 0..map_size.x {
            // TODO: due to coordinate conversion, this is wrong for
            // every chunk but the one the player is in.
            //
            // Player always starts at an empty space:
            let kind = if player_pos == (x, y) {
                TileKind::Empty
            } else {
                choices
                    .choose_weighted(rng, |item| item.1)
                    .map(|result| result.0)
                    .unwrap_or(TileKind::Empty)
            };

            let mut tile = Tile::new(kind);
            if tile.kind == TileKind::Tree {
                let options = [color::tree_1, color::tree_2, color::tree_3];
                tile.fg_color = *options.choose(throwavay_rng).unwrap();
            }

            result.push((Point::new(x, y), tile));
        }
    }
    result
}

fn generate_monsters<R: Rng>(rng: &mut R, map: &[(Point, Tile)]) -> Vec<Monster> {
    let monster_count = 5;
    let monster_chance = 30;
    let options = [
        (None, 1000 - monster_chance),
        (Some(Kind::Anxiety), monster_chance / monster_count),
        (Some(Kind::Depression), monster_chance / monster_count),
        (Some(Kind::Hunger), monster_chance / monster_count),
        (Some(Kind::Shadows), monster_chance / monster_count),
        (Some(Kind::Voices), monster_chance / monster_count),
        (Some(Kind::Npc), 2),
    ];

    let mut result = vec![];
    for &(pos, tile) in map.iter() {
        if tile.kind != TileKind::Empty {
            continue;
        }
        let kind = options
            .choose_weighted(rng, |item| item.1)
            .map(|result| result.0)
            .unwrap_or(None);
        if let Some(kind) = kind {
            let mut monster = Monster::new(kind, pos);
            if kind == Kind::Npc {
                use crate::color;
                use crate::monster::CompanionBonus::*;
                let bonus = rng.gen();
                monster.companion_bonus = Some(bonus);
                monster.color = match bonus {
                    DoubleWillGrowth => color::npc_will,
                    HalveExhaustion => color::npc_mind,
                    ExtraActionPoint => color::npc_speed,
                    Victory => unreachable!(),
                };
            };
            result.push(monster);
        }
    }
    result
}

fn new_item<R: Rng>(kind: item::Kind, rng: &mut R) -> Item {
    use crate::item::Kind::*;
    match kind {
        Dose => {
            let mut item = formula::DOSE_PREFAB;
            if let Modifier::Intoxication {
                ref mut state_of_mind,
                ..
            } = item.modifier
            {
                *state_of_mind += formula::DOSE_MIND_VARIANCE.random(rng);
            };
            item
        }
        StrongDose => {
            let mut item = formula::STRONG_DOSE_PREFAB;
            if let Modifier::Intoxication {
                ref mut state_of_mind,
                ..
            } = item.modifier
            {
                *state_of_mind += formula::STRONG_DOSE_MIND_VARIANCE.random(rng);
            };
            item
        }
        CardinalDose => {
            let mut item = formula::CARDINAL_DOSE_PREFAB;
            if let Modifier::Intoxication {
                ref mut state_of_mind,
                ..
            } = item.modifier
            {
                *state_of_mind += formula::CARDINAL_DOSE_MIND_VARIANCE.random(rng);
            };
            item
        }
        DiagonalDose => {
            let mut item = formula::DIAGONAL_DOSE_PREFAB;
            if let Modifier::Intoxication {
                ref mut state_of_mind,
                ..
            } = item.modifier
            {
                *state_of_mind += formula::DIAGONAL_DOSE_MIND_VARIANCE.random(rng);
            };
            item
        }
        Food => formula::FOOD_PREFAB,
    }
}

fn generate_items<R: Rng>(rng: &mut R, map: &[(Point, Tile)]) -> Vec<(Point, Item)> {
    use crate::item::Kind::*;
    let options = [
        (None, 1000),
        (Some(Dose), 8),
        (Some(StrongDose), 3),
        (Some(CardinalDose), 2),
        (Some(DiagonalDose), 2),
        (Some(Food), 5),
    ];

    // NOTE: this calculates how many items we need to place. It
    // calculates the baseline number of empty tiles and the average
    // chance of an item appearing on an empty tile. Then we ensure we
    // actually hit that number.
    let item_count: i32 = options
        .iter()
        .filter(|(kind, _)| kind.is_some())
        .map(|(_, count)| count)
        .sum();
    let total_count: i32 = options.iter().map(|i| i.1).sum();
    let item_percentage = item_count as f32 / total_count as f32;
    let empty_tile_count = (map.len() as f32 * (1.0 - formula::CHUNK_BASELINE_DENSITY)).ceil();

    let mut items_to_place = (empty_tile_count * item_percentage) as i32;
    let mut result = vec![];
    for &(pos, tile) in map.iter().cycle() {
        if items_to_place <= 0 {
            break;
        }
        match tile.kind {
            TileKind::Tree => {
                // Occupied tile, do nothing.
            }
            TileKind::Empty => {
                let kind = options
                    .choose_weighted(rng, |item| item.1)
                    .map(|result| result.0)
                    .unwrap_or(None);
                if let Some(kind) = kind {
                    result.push((pos, new_item(kind, rng)));
                    items_to_place -= 1;
                }
            }
        }
    }
    result
}

pub fn generate<R: Rng, G: Rng>(
    rng: &mut R,
    throwavay_rng: &mut G,
    size: Point,
    player: Point,
) -> GeneratedWorld {
    let map = generate_map(rng, throwavay_rng, size, player);
    let monsters = generate_monsters(rng, &map);
    let items = generate_items(rng, &map);
    (map, monsters, items)
}
