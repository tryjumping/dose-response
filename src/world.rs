use std::rand;
use std::rand::Rng;

// TODO: looks like we want to namespace these some more:
use components::{AcceptsUserInput,Attributes, Addiction, AnxietyKillCounter, AI,
                 AttributeModifier, Corpse, Dose, ExplosionEffect,
                 Kill, ModifyAttributes, Monster, Panic, Infinite, Stun,
                 Exploration, Position, Tile, Turn, Sec};
use components::{Anxiety, Depression, Hunger, Shadows, Voices};  // monster types
use components::{Computer, Player};  // sides
use components::{Background, Edible, Explored, Pickable, Solid};  // flags
use components::ai;
use emhyr::{World, Components, Entity};
use engine::Color;
use entity_util;
use world_gen;
use point;


pub fn populate_world<T: Rng>(world: &mut World,
                              world_size: (int, int),
                              player_pos: Position,
                              rng: &mut T,
                              generate: fn(&mut T, int, int) -> Vec<(int, int, world_gen::WorldItem)>) {
    let near_player = |x, y| point::tile_distance(player_pos, (x, y)) < 6;
    let pos_offset = [-4, -3, -2, -1, 1, 2, 3, 4];
    let initial_dose_pos = (player_pos.x + *rng.choose(pos_offset).unwrap(),
                            player_pos.y + *rng.choose(pos_offset).unwrap());
    let mut initial_foods_pos = Vec::<(int, int)>::new();
    for _ in range(0, rng.gen_range::<uint>(1, 4)) {
            let pos = (player_pos.x + *rng.choose(pos_offset).unwrap(),
                       player_pos.y + *rng.choose(pos_offset).unwrap());
            initial_foods_pos.push(pos);
    };
    let (width, height) = world_size;
    let map = generate(rng, width, height);
    for &(x, y, item) in map.iter() {
        let bg = world.new_entity();
        world.cs.set(Position{x: x, y: y}, bg);
        world.cs.set(Background, bg);
        let explored = point::distance((x, y), (player_pos.x, player_pos.y)) < 6f32;
        if explored {
            world.cs.set(Explored, bg);
        }
        let item = if (x, y) == (player_pos.x, player_pos.y) {
            world_gen::Empty
        } else {
            item
        };
        let is_initial_food = initial_foods_pos.iter().any(|&p| p == (x, y));
        let item = if (x, y) == initial_dose_pos {
            world_gen::Dose
        } else if is_initial_food {
            world_gen::Food
        } else {
            item
        };
        if item == world_gen::Tree {
            world.cs.set(Tile{level: 0, glyph: item.to_glyph(), color: item.to_color()}, bg);
            world.cs.set(Solid, bg);
        } else { // put an empty item as the background
            world.cs.set(Tile{level: 0, glyph: world_gen::Empty.to_glyph(), color: world_gen::Empty.to_color()}, bg);
        }
        if near_player(x, y) && ((x, y) != initial_dose_pos) && !is_initial_food {
            continue
        };
        if item != world_gen::Tree && item != world_gen::Empty {
            let e = world.new_entity();
            world.cs.set(Position{x: x, y: y}, e);
            let mut tile_level = 1;
            if item.is_monster() {
                let behaviour = match item {
                    world_gen::Hunger => ai::Pack,
                    _ => ai::Individual,
                };
                world.cs.set(AI{behaviour: behaviour, state: ai::Idle}, e);
                let max_ap = if item == world_gen::Depression { 2 } else { 1 };
                world.cs.set(Turn{side: Computer,
                                   ap: 0,
                                   max_ap: max_ap,
                                   spent_this_tick: 0,
                    }, e);
                world.cs.set(Solid, e);
                match item {
                    world_gen::Anxiety => {
                        world.cs.set(Monster{kind: Anxiety}, e);
                        world.cs.set(ModifyAttributes, e);
                        world.cs.set(AttributeModifier{state_of_mind: 0, will: -1}, e);
                    }
                    world_gen::Depression => {
                        world.cs.set(Monster{kind: Depression}, e);
                        world.cs.set(Kill, e)
                    },
                    world_gen::Hunger => {
                        world.cs.set(Monster{kind: Hunger}, e);
                        world.cs.set(ModifyAttributes, e);
                        world.cs.set(AttributeModifier{state_of_mind: -20, will: 0}, e)
                    }
                    world_gen::Voices => {
                        world.cs.set(Monster{kind: Voices}, e);
                        world.cs.set(Stun{duration: 4}, e)
                    },
                    world_gen::Shadows => {
                        world.cs.set(Monster{kind: Shadows}, e);
                        world.cs.set(Panic{duration: 4}, e)
                    },
                    _ => unreachable!(),
                };
                tile_level = 2;
            } else if item == world_gen::Dose {
                world.cs.set(Dose{tolerance_modifier: 1, resist_radius: 2}, e);
                entity_util::set_color_animation_loop(
                    &mut world.cs, e, item.to_color(), col::dose_glow,
                    Infinite, Sec(0.6));
                world.cs.set(AttributeModifier{
                        state_of_mind: 72 + rng.gen_range(-5i, 6),
                        will: 0,
                    }, e);
                world.cs.set(ExplosionEffect{radius: 4}, e);
                if (x, y) == initial_dose_pos {
                    world.cs.set(Explored, e);
                }
            } else if item == world_gen::StrongDose {
                world.cs.set(Dose{tolerance_modifier: 2, resist_radius: 3}, e);
                world.cs.set(AttributeModifier{
                        state_of_mind: 130 + rng.gen_range(-15i, 16),
                        will: 0,
                    }, e);
                entity_util::set_color_animation_loop(
                    &mut world.cs, e, item.to_color(), col::dose_glow,
                    Infinite, Sec(0.5));
                world.cs.set(ExplosionEffect{radius: 6}, e);
            } else if item == world_gen::Food {
                world.cs.set(ExplosionEffect{radius: 2}, e);
                world.cs.set(Pickable, e);
                world.cs.set(Edible, e);
                if is_initial_food {
                    world.cs.set(Explored, e);
                }
            }
            world.cs.set(Tile{level: tile_level, glyph: item.to_glyph(), color: item.to_color()}, e);
        }
    }
}

pub fn create_player(cs: &mut Components, player: Entity) {
    cs.set(AcceptsUserInput, player);
    cs.set(Kill, player);
    cs.set(Attributes{state_of_mind: 20, will: 2}, player);
    cs.set(Addiction{
            tolerance: 0,
            drop_per_turn: 1,
            last_turn: 1,
        }, player);
    cs.set(AnxietyKillCounter{
            count: 0,
            threshold: 10}, player);
    cs.set(Exploration{radius: 5}, player);
    cs.set(Explored, player);
    cs.set(Tile{level: 2, glyph: '@', color: col::player}, player);
    cs.set(Corpse{
            glyph: '&',
            color: col::dead_player,
            solid: true}, player);
    cs.set(Turn{side: Player,
                            ap: 1,
                            max_ap: 1,
                            spent_this_tick: 0,
        }, player);
    cs.set(Solid, player);
}


// TODO: adding this dummy trait so we can implemen these methods here instead
// of in the world_gen module where the struct is defined. A recent Rust upgrade
// broke that and I'm not currently sure if that's to stay or not.
//
// We'll see once things settle down.
trait MyWorldItemDummyTrait {
    fn to_glyph(self) -> char;
    fn to_color(self) -> Color;
    fn is_monster(self) -> bool;
}

impl MyWorldItemDummyTrait for world_gen::WorldItem {
    fn to_glyph(self) -> char {
        match self {
            world_gen::Empty => '.',
            world_gen::Tree => '#',
            world_gen::Dose => 'i',
            world_gen::StrongDose => 'I',
            world_gen::Food => '%',
            world_gen::Anxiety => 'a',
            world_gen::Depression => 'D',
            world_gen::Hunger => 'h',
            world_gen::Voices => 'v',
            world_gen::Shadows => 'S',
        }
    }

    fn to_color(self) -> Color {
        match self {
            world_gen::Empty => col::empty_tile,
            world_gen::Tree => *rand::task_rng().choose(&[col::tree_1, col::tree_2, col::tree_3]).unwrap(),
            world_gen::Dose => col::dose,
            world_gen::StrongDose => col::dose,
            world_gen::Food => col::food,

            world_gen::Anxiety => col::anxiety,
            world_gen::Depression => col::depression,
            world_gen::Hunger => col::hunger,
            world_gen::Voices => col::voices,
            world_gen::Shadows => col::shadows,
        }
    }

    fn is_monster(self) -> bool {
        match self {
            world_gen::Anxiety |
            world_gen::Depression |
            world_gen::Hunger |
            world_gen::Voices |
            world_gen::Shadows => true,
            _ => false,
        }
    }
}

pub mod col {
    use engine::Color;

    pub static background: Color = Color{r: 0, g: 0, b: 0};
    pub static dim_background: Color = Color{r: 30, g: 30, b: 30};
    pub static anxiety: Color = Color{r: 191,g: 0,b: 0};
    pub static depression: Color = Color{r: 111,g: 63,b: 255};
    pub static hunger: Color = Color{r: 127,g: 101,b: 63};
    pub static voices: Color = Color{r: 95,g: 95,b: 95};
    pub static shadows: Color = Color{r: 95,g: 95,b: 95};
    pub static player: Color = Color{r: 255,g: 255,b: 255};
    pub static dead_player: Color = Color{r: 80, g: 80, b: 80};
    pub static empty_tile: Color = Color{r: 223,g: 223,b: 223};
    pub static dose: Color = Color{r: 114,g: 126,b: 255};
    pub static dose_glow: Color = Color{r: 15, g: 255, b: 243};
    pub static food: Color = Color{r: 148, g: 113, b: 0};
    pub static tree_1: Color = Color{r: 0,g: 191,b: 0};
    pub static tree_2: Color = Color{r: 0,g: 255,b: 0};
    pub static tree_3: Color = Color{r: 63,g: 255,b: 63};
    pub static high: Color = Color{r: 58, g: 217, b: 183};
    pub static high_to: Color = Color{r: 161, g: 39, b: 113};
}
