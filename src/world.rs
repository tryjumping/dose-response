use std::rand::Rng;

use components::*;
use engine::Color;
use entity_manager::EntityManager;
use map;
use map::Map;
use world_gen;


pub fn populate_world<T: Rng>(ecm: &mut EntityManager<GameObject>,
                              map: &mut Map,
                              rng: &mut T,
                              generate: &fn(&mut T, uint, uint) -> ~[(int, int, world_gen::WorldItem)]) {
    let world = generate(rng, map.width, map.height);
    for &(x, y, item) in world.iter() {
        let mut bg = GameObject::new();
        bg.position = Some(Position{x: x, y: y});
        bg.background = Some(Background);
        if item == world_gen::Tree {
            bg.tile = Some(Tile{level: 0, glyph: item.to_glyph(), color: item.to_color()});
            bg.solid = Some(Solid);
        } else { // put an empty item as the background
            bg.tile = Some(Tile{level: 0, glyph: world_gen::Empty.to_glyph(), color: world_gen::Empty.to_color()});
        }
        ecm.add(bg);
        if item != world_gen::Tree && item != world_gen::Empty {
            let mut e = GameObject::new();
            e.position = Some(Position{x: x, y: y});
            let mut tile_level = 1;
            if item.is_monster() {
                let behaviour = match item {
                    world_gen::Hunger => ai::Pack,
                    _ => ai::Individual,
                };
                e.ai = Some(AI{behaviour: behaviour, state: ai::Idle});
                let max_ap = if item == world_gen::Depression { 2 } else { 1 };
                e.turn = Some(Turn{side: Computer,
                                   ap: 0,
                                   max_ap: max_ap,
                                   spent_this_tick: 0,
                    });
                e.solid = Some(Solid);
                match item {
                    world_gen::Anxiety => {
                        e.monster = Some(Monster{kind: Anxiety});
                        e.attack_type = Some(ModifyAttributes);
                        e.attribute_modifier = Some(
                            AttributeModifier{state_of_mind: 0, will: -1});
                    }
                    world_gen::Depression => {
                        e.monster = Some(Monster{kind: Depression});
                        e.attack_type = Some(Kill)
                    },
                    world_gen::Hunger => {
                        e.monster = Some(Monster{kind: Hunger});
                        e.attack_type = Some(ModifyAttributes);
                        e.attribute_modifier = Some(
                            AttributeModifier{state_of_mind: -20, will: 0})
                    }
                    world_gen::Voices => {
                        e.monster = Some(Monster{kind: Voices});
                        e.attack_type = Some(Stun{duration: 4})
                    },
                    world_gen::Shadows => {
                        e.monster = Some(Monster{kind: Shadows});
                        e.attack_type = Some(Panic{duration: 4})
                    },
                    _ => unreachable!(),
                };
                tile_level = 2;
            } else if item == world_gen::Dose {
                e.dose = Some(Dose{tolerance_modifier: 1, resist_radius: 2});
                e.attribute_modifier = Some(AttributeModifier{
                        state_of_mind: 40 + rng.gen_integer_range(-10, 11),
                        will: 0,
                    });
                e.explosion_effect = Some(ExplosionEffect{radius: 4});
            } else if item == world_gen::StrongDose {
                e.dose = Some(Dose{tolerance_modifier: 2, resist_radius: 3});
                e.attribute_modifier = Some(AttributeModifier{
                        state_of_mind: 90 + rng.gen_integer_range(-15, 16),
                        will: 0,
                    });
                e.explosion_effect = Some(ExplosionEffect{radius: 6});
            }
            e.tile = Some(Tile{level: tile_level, glyph: item.to_glyph(), color: item.to_color()});
            ecm.add(e);
        }
    }

    // Initialise the map's walkability data
    for (e, id) in ecm.iter() {
        match e.position {
            Some(Position{x, y}) => {
                let walkable = match e.solid {
                    Some(_) => map::Solid,
                    None => map::Walkable,
                };
                match e.background {
                    Some(_) => map.set_walkability((x, y), walkable),
                    None => map.place_entity(*id, (x, y), walkable),
                }
            },
            None => (),
        }
    }
}

pub fn player_entity() -> GameObject {
    let mut player = GameObject::new();
    player.accepts_user_input = Some(AcceptsUserInput);
    player.attack_type = Some(Kill);
    player.attributes = Some(Attributes{state_of_mind: 20, will: 2});
    player.addiction = Some(Addiction{
            tolerance: 0,
            drop_per_turn: 1,
            last_turn: 1,
        });
    player.anxiety_kill_counter = Some(AnxietyKillCounter{
            count: 0,
            threshold: 10});
    player.position = Some(Position{x: 10, y: 20});
    player.tile = Some(Tile{level: 2, glyph: '@', color: col::player});
    player.turn = Some(Turn{side: Player,
                            ap: 0,
                            max_ap: 1,
                            spent_this_tick: 0,
        });
    player.solid = Some(Solid);
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
