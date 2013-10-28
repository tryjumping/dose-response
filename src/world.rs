use std::rand::Rng;

use components::*;
use engine::Color;
use map;
use map::Map;
use world_gen;
use systems::ai::distance;


pub fn populate_world<T: Rng>(ecm: &mut ComponentManager,
                              map: &mut Map,
                              player_pos: Position,
                              rng: &mut T,
                              generate: &fn(&mut T, uint, uint) -> ~[(int, int, world_gen::WorldItem)]) {
    let near_player = |x, y| distance(&player_pos, &Position{x: x, y: y}) < 6;
    let pos_offset = [-4, -3, -2, -1, 1, 2, 3, 4];
    let initial_dose_pos = (player_pos.x + rng.choose(pos_offset),
                            player_pos.y + rng.choose(pos_offset));
    let world = generate(rng, map.width, map.height);
    for &(x, y, item) in world.iter() {
        let bg = ecm.new_entity();
        ecm.set_position(bg, Position{x: x, y: y});
        ecm.set_background(bg, Background);
        let item = if (x, y) == (player_pos.x, player_pos.y) {
            world_gen::Empty
        } else {
            item
        };
        let item = if (x, y) == initial_dose_pos {
            world_gen::Dose
        } else {
            item
        };
        if item == world_gen::Tree {
            ecm.set_tile(bg, Tile{level: 0, glyph: item.to_glyph(), color: item.to_color()});
            ecm.set_solid(bg, Solid);
        } else { // put an empty item as the background
            ecm.set_tile(bg, Tile{level: 0, glyph: world_gen::Empty.to_glyph(), color: world_gen::Empty.to_color()});
        }
        if near_player(x, y) && ((x, y) != initial_dose_pos) {
            loop
        };
        if item != world_gen::Tree && item != world_gen::Empty {
            let e = ecm.new_entity();
            ecm.set_position(e, Position{x: x, y: y});
            let mut tile_level = 1;
            if item.is_monster() {
                let behaviour = match item {
                    world_gen::Hunger => ai::Pack,
                    _ => ai::Individual,
                };
                ecm.set_ai(e, AI{behaviour: behaviour, state: ai::Idle});
                let max_ap = if item == world_gen::Depression { 2 } else { 1 };
                ecm.set_turn(e, Turn{side: Computer,
                                   ap: 0,
                                   max_ap: max_ap,
                                   spent_this_tick: 0,
                    });
                ecm.set_solid(e, Solid);
                match item {
                    world_gen::Anxiety => {
                        ecm.set_monster(e, Monster{kind: Anxiety});
                        ecm.set_attack_type(e, ModifyAttributes);
                        ecm.set_attribute_modifier(e,
                            AttributeModifier{state_of_mind: 0, will: -1});
                    }
                    world_gen::Depression => {
                        ecm.set_monster(e, Monster{kind: Depression});
                        ecm.set_attack_type(e, Kill)
                    },
                    world_gen::Hunger => {
                        ecm.set_monster(e, Monster{kind: Hunger});
                        ecm.set_attack_type(e, ModifyAttributes);
                        ecm.set_attribute_modifier(e,
                            AttributeModifier{state_of_mind: -20, will: 0})
                    }
                    world_gen::Voices => {
                        ecm.set_monster(e, Monster{kind: Voices});
                        ecm.set_attack_type(e, Stun{duration: 4})
                    },
                    world_gen::Shadows => {
                        ecm.set_monster(e, Monster{kind: Shadows});
                        ecm.set_attack_type(e, Panic{duration: 4})
                    },
                    _ => unreachable!(),
                };
                tile_level = 2;
            } else if item == world_gen::Dose {
                ecm.set_dose(e, Dose{tolerance_modifier: 1, resist_radius: 2});
                ecm.set_attribute_modifier(e, AttributeModifier{
                        state_of_mind: 40 + rng.gen_integer_range(-10, 11),
                        will: 0,
                    });
                ecm.set_explosion_effect(e, ExplosionEffect{radius: 4});
            } else if item == world_gen::StrongDose {
                ecm.set_dose(e, Dose{tolerance_modifier: 2, resist_radius: 3});
                ecm.set_attribute_modifier(e, AttributeModifier{
                        state_of_mind: 90 + rng.gen_integer_range(-15, 16),
                        will: 0,
                    });
                ecm.set_explosion_effect(e, ExplosionEffect{radius: 6});
            }
            ecm.set_tile(e, Tile{level: tile_level, glyph: item.to_glyph(), color: item.to_color()});
        }
    }

    // Initialise the map's walkability data
    for e in ecm.iter() {
        let Position{x, y} =  ecm.get_position(e);
        let walkable = match ecm.has_solid(e) {
            true => map::Solid,
            false => map::Walkable,
        };
        match ecm.has_background(e) {
            true => map.set_walkability((x, y), walkable),
            false => map.place_entity(*e, (x, y), walkable),
        }
    }
}

pub fn player_entity(ecm: &mut ComponentManager) -> ID {
    let player = ecm.new_entity();
    ecm.set_accepts_user_input(player, AcceptsUserInput);
    ecm.set_attack_type(player, Kill);
    ecm.set_attributes(player, Attributes{state_of_mind: 20, will: 2});
    ecm.set_addiction(player, Addiction{
            tolerance: 0,
            drop_per_turn: 1,
            last_turn: 1,
        });
    ecm.set_anxiety_kill_counter(player, AnxietyKillCounter{
            count: 0,
            threshold: 10});
    ecm.set_position(player, Position{x: 10, y: 20});
    ecm.set_tile(player, Tile{level: 2, glyph: '@', color: col::player});
    ecm.set_turn(player, Turn{side: Player,
                            ap: 0,
                            max_ap: 1,
                            spent_this_tick: 0,
        });
    ecm.set_solid(player, Solid);
    return player;
}


impl world_gen::WorldItem {
    fn to_glyph(self) -> char {
        match self {
            world_gen::Empty => '.',
            world_gen::Tree => '#',
            world_gen::Dose => 'i',
            world_gen::StrongDose => 'I',
            world_gen::Anxiety => 'a',
            world_gen::Depression => 'D',
            world_gen::Hunger => 'h',
            world_gen::Voices => 'v',
            world_gen::Shadows => 's',
        }
    }

    fn to_color(self) -> Color {
        match self {
            world_gen::Empty => col::empty_tile,
            world_gen::Tree => col::tree_1,
            world_gen::Dose => col::dose,
            world_gen::StrongDose => col::dose,

            world_gen::Anxiety => col::anxiety,
            world_gen::Depression => col::depression,
            world_gen::Hunger => col::hunger,
            world_gen::Voices => col::voices,
            world_gen::Shadows => col::shadows,
        }
    }

    fn is_solid(self) -> bool {
        match self {
            world_gen::Empty | world_gen::Dose | world_gen::StrongDose => false,
            _ => true,
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

mod col {
    use engine::Color;

    pub static background: Color = Color(0, 0, 0);
    pub static dim_background: Color = Color(15, 15, 15);
    pub static foreground: Color = Color(255, 255, 255);
    pub static anxiety: Color = Color(191,0,0);
    pub static depression: Color = Color(111,63,255);
    pub static hunger: Color = Color(127,101,63);
    pub static voices: Color = Color(95,95,95);
    pub static shadows: Color = Color(95,95,95);
    pub static player: Color = Color(255,255,255);
    pub static empty_tile: Color = Color(223,223,223);
    pub static dose: Color = Color(114,184,255);
    pub static dose_glow: Color = Color(0,63,47);
    pub static tree_1: Color = Color(0,191,0);
    pub static tree_2: Color = Color(0,255,0);
    pub static tree_3: Color = Color(63,255,63);
}
