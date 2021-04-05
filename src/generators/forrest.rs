use crate::{
    formula,
    generators::GeneratedWorld,
    graphic::Graphic,
    item::{self, Item},
    level::{Tile, TileKind},
    monster::{Kind, Monster},
    palette,
    player::Modifier,
    point::Point,
    random::Random,
    state::Challenge,
};

// TODO: Instead of `map_size`, use a Rectangle with the world
// positions here. We want to expose the non-world coordinates in as
// few places as possible.
fn generate_map(
    rng: &mut Random,
    throwavay_rng: &mut Random,
    map_size: Point,
    player_pos: Point,
) -> Vec<(Point, Tile)> {
    assert!(formula::CHUNK_DENSITY_VARIABILITY.0 < formula::CHUNK_DENSITY_VARIABILITY.1);
    assert!(formula::CHUNK_BASELINE_DENSITY + formula::CHUNK_DENSITY_VARIABILITY.0 > 0.0);
    assert!(formula::CHUNK_BASELINE_DENSITY + formula::CHUNK_DENSITY_VARIABILITY.1 < 1.0);

    let density = formula::CHUNK_BASELINE_DENSITY
        + crate::graphics::lerp_f32(
            formula::CHUNK_DENSITY_VARIABILITY.0,
            formula::CHUNK_DENSITY_VARIABILITY.1,
            rng.rand_float(),
        );
    let occupied_count = (density * 100.0) as u32;
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
                *rng.choose_weighted(&choices).unwrap_or(&TileKind::Empty)
            };

            let mut tile = Tile::new(kind);
            match tile.kind {
                TileKind::Tree => {
                    tile.color_index =
                        throwavay_rng.range_inclusive(0, palette::TREE_COUNT as i32 - 1) as usize;

                    let graphic_options = [
                        Graphic::Tree1,
                        Graphic::Tree2,
                        Graphic::Tree3,
                        Graphic::Tree4,
                        Graphic::Tree5,
                        Graphic::Tree6,
                        Graphic::Tree7,
                        Graphic::Tree8,
                        Graphic::Tree9,
                        Graphic::Tree10,
                    ];
                    tile.graphic =
                        *throwavay_rng.choose_with_fallback(&graphic_options, &Graphic::Tree1);
                }
                TileKind::Empty => {
                    let options = [
                        Graphic::Ground2,
                        Graphic::Ground3,
                        Graphic::Ground5,
                        Graphic::Twigs1,
                        Graphic::Twigs2,
                        Graphic::Twigs3,
                        Graphic::Twigs4,
                        Graphic::Twigs5,
                        Graphic::Twigs6,
                        Graphic::Twigs7,
                        Graphic::Twigs8,
                        Graphic::Twigs9,
                        Graphic::Twigs10,
                        Graphic::Grass1,
                        Graphic::Grass2,
                        Graphic::Grass3,
                        Graphic::Grass4,
                        Graphic::Grass5,
                        Graphic::Grass6,
                        Graphic::Grass7,
                        Graphic::Grass8,
                        Graphic::Grass9,
                        Graphic::Leaves1,
                        Graphic::Leaves3,
                        Graphic::Leaves4,
                        Graphic::Leaves5,
                    ];
                    let graphic = *throwavay_rng.choose_with_fallback(&options, &Graphic::Ground2);
                    tile.graphic = graphic;
                }
            };

            result.push((Point::new(x, y), tile));
        }
    }
    result
}

fn generate_monsters(
    rng: &mut Random,
    map: &[(Point, Tile)],
    challenge: Challenge,
) -> Vec<Monster> {
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
        let kind = *rng.choose_weighted(&options).unwrap_or(&None);
        if let Some(kind) = kind {
            let mut monster = Monster::new(kind, pos, challenge);
            if kind == Kind::Npc {
                let bonus = crate::monster::CompanionBonus::random(rng);
                monster.companion_bonus = Some(bonus);
            };
            result.push(monster);
        }
    }
    result
}

fn new_item(kind: item::Kind, rng: &mut Random) -> Item {
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
        Food => {
            let mut item = formula::FOOD_PREFAB;
            item.graphic = *rng.choose_with_fallback(
                &[
                    Graphic::FoodAcornWide,
                    Graphic::FoodAcornThin,
                    Graphic::FoodCarrotWide,
                    Graphic::FoodCarrotSideways,
                    Graphic::FoodCarrotThin,
                    Graphic::FoodTurnipSmallLeaves,
                    Graphic::FoodTurnipBigLeaves,
                    Graphic::FoodTurnipHeart,
                    Graphic::FoodStriped,
                ],
                &Graphic::FoodAcornWide,
            );
            item
        }
    }
}

fn generate_items(rng: &mut Random, map: &[(Point, Tile)]) -> Vec<(Point, Item)> {
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
    let item_count: u32 = options
        .iter()
        .filter(|(kind, _)| kind.is_some())
        .map(|(_, count)| count)
        .sum();
    let total_count = options.iter().map(|i| i.1).sum::<u32>() as i32;
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
                let kind = *rng.choose_weighted(&options).unwrap_or(&None);
                if let Some(kind) = kind {
                    result.push((pos, new_item(kind, rng)));
                    items_to_place -= 1;
                }
            }
        }
    }
    result
}

pub fn generate(
    rng: &mut Random,
    throwavay_rng: &mut Random,
    size: Point,
    player: Point,
    challenge: Challenge,
) -> GeneratedWorld {
    let map = generate_map(rng, throwavay_rng, size, player);
    let monsters = generate_monsters(rng, &map, challenge);
    let items = generate_items(rng, &map);
    (map, monsters, items)
}
