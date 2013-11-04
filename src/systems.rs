macro_rules! ensure_components(
    ($ecm:expr, $entity:expr, $($component:ident),+) => (
        if !$ecm.has_entity($entity) || $(!$ecm.has_component($entity, concat_idents!(t, $component)))||+ {return}
    )
)

pub mod turn_tick_counter {
    use components::*;
    use super::super::Resources;

    pub fn system(e: ID,
                  ecm: &mut ComponentManager,
                  res: &mut Resources) {
        ensure_components!(ecm, e, Turn);
        let turn = ecm.get_turn(e);
        if turn.side == res.side {
            ecm.set_turn(e, Turn{spent_this_tick: 0, .. turn});
        }
    }
}

pub mod input {
    use components::*;
    use self::commands::*;
    use super::super::Resources;

    pub mod commands {
        #[deriving(Rand, ToStr)]
        pub enum Command {
            N, E, S, W, NE, NW, SE, SW,
        }
    }

    impl FromStr for Command {
        fn from_str(name: &str) -> Option<Command> {
            match name {
                "N" => Some(N),
                "E" => Some(E),
                "S" => Some(S),
                "W" => Some(W),
                "NE" => Some(NE),
                "NW" => Some(NW),
                "SE" => Some(SE),
                "SW" => Some(SW),
                _ => None,
            }
        }
    }

    pub fn system(e: ID,
                  ecm: &mut ComponentManager,
                  res: &mut Resources) {
        ensure_components!(ecm, e, AcceptsUserInput, Position);
        if res.side != Player {return}

        let pos = ecm.get_position(e);
        match res.commands.pop_front() {
            Some(command) => {
                res.command_logger.log(command);
                let dest = match command {
                    N => Destination{x: pos.x, y: pos.y-1},
                    S => Destination{x: pos.x, y: pos.y+1},
                    W => Destination{x: pos.x-1, y: pos.y},
                    E => Destination{x: pos.x+1, y: pos.y},

                    NW => Destination{x: pos.x-1, y: pos.y-1},
                    NE => Destination{x: pos.x+1, y: pos.y-1},
                    SW => Destination{x: pos.x-1, y: pos.y+1},
                    SE => Destination{x: pos.x+1, y: pos.y+1},
                };
                ecm.set_destination(e, dest);
            },
            None => (),
        }
    }
}


pub mod leave_area {
    use components::*;
    use map::Map;
    use world_gen;
    use world;
    use super::super::Resources;

    pub fn system(e: ID,
                  ecm: &mut ComponentManager,
                  res: &mut Resources) {
        ensure_components!(ecm, e, Position, Destination);
        if e != res.player_id {return}
        let dest = ecm.get_destination(e);
        let left_map_boundaries = (dest.x < 0 || dest.y < 0 ||
                                   dest.x >= res.map.width ||
                                   dest.y >= res.map.height);
        if left_map_boundaries {
            let player_entity = ecm.take_out(res.player_id);
            ecm.remove_all_entities();
            let player_id = ecm.add_entity(player_entity);
            res.player_id = player_id;
            // The player starts in the middle of the map with no pending
            // actions:
            ecm.set_position(player_id, Position{
                    x: (res.map.width / 2) as int,
                    y: (res.map.height / 2) as int,
                });
            ecm.remove_bump(player_id);
            ecm.remove_attack_target(player_id);
            ecm.remove_destination(player_id);
            res.map = Map::new(res.map.width, res.map.height);
            let player_pos = ecm.get_position(player_id);
            world::populate_world(ecm,
                                  &mut res.map,
                                  player_pos,
                                  &mut res.rng,
                                  world_gen::forrest);
            // TODO: We don't want the curret tick to continue after we've messed with
            // the game state. Signal the main loop to abort it early.
        }
    }
}

pub mod ai {
    use std::rand::Rng;
    use components::*;
    use components;
    use map::Map;
    use std::num::{abs, max};
    use super::super::Resources;


    pub fn distance(p1: &Position, p2: &Position) -> int {
        max(abs(p1.x - p2.x), abs(p1.y - p2.y))
    }

    pub fn random_neighbouring_position<T: Rng>(rng: &mut T,
                                                pos: Position,
                                                map: &Map) -> (int, int) {
        let neighbors = [
            (pos.x, pos.y-1),
            (pos.x, pos.y+1),
            (pos.x-1, pos.y),
            (pos.x+1, pos.y),
            (pos.x-1, pos.y-1),
            (pos.x+1, pos.y-1),
            (pos.x-1, pos.y+1),
            (pos.x+1, pos.y+1),
            ];
        let mut walkables: ~[(int, int)] = ~[];
        for &p in neighbors.iter() {
            if map.is_walkable(p) { walkables.push(p) }
        }
        if walkables.is_empty() {
            (pos.x, pos.y)  // Nowhere to go
        } else {
            rng.choose(walkables)
        }
    }

    pub fn entity_blocked(pos: Position, map: &Map) -> bool {
        let neighbors = [
            (pos.x, pos.y-1),
            (pos.x, pos.y+1),
            (pos.x-1, pos.y),
            (pos.x+1, pos.y),
            (pos.x-1, pos.y-1),
            (pos.x+1, pos.y-1),
            (pos.x-1, pos.y+1),
            (pos.x+1, pos.y+1),
            ];
        !do neighbors.iter().any |&neighbor_pos| {
            map.is_walkable(neighbor_pos)
        }
    }

    fn individual_behaviour<T: Rng>(e: ID,
                                    ecm: &mut ComponentManager,
                                    rng: &mut T,
                                    map: &Map,
                                    player_pos: Position) -> Destination {
        let pos = ecm.get_position(e);
        let player_distance = distance(&pos, &player_pos);
        let ai = ecm.get_ai(e);
        match player_distance {
            dist if dist < 5 => {
                ecm.set_ai(e, AI{state: components::ai::Aggressive, .. ai});
            }
            dist if dist > 8 => {
                ecm.set_ai(e, AI{state: components::ai::Idle, .. ai});
            }
            _ => {}
        }
        match ecm.get_ai(e).state {
            components::ai::Aggressive => {
                Destination{x: player_pos.x, y: player_pos.y}
            }
            components::ai::Idle => {
                match random_neighbouring_position(rng, pos, map) {
                    (x, y) => Destination{x: x, y: y}
                }
            }
        }
    }

    fn hunting_pack_behaviour<T: Rng>(e: ID,
                                      ecm: &mut ComponentManager,
                                      rng: &mut T,
                                      map: &Map,
                                      player_pos: Position) -> Destination {
        let pos = ecm.get_position(e);
        let player_distance = distance(&pos, &player_pos);
        if player_distance < 4 {
            let ai = ecm.get_ai(e);
            ecm.set_ai(e, AI{state: components::ai::Aggressive, .. ai});
        }
        match ecm.get_ai(e).state {
            components::ai::Aggressive => {
                let r = 8;
                for x in range(pos.x - r, pos.x + r) {
                    for y in range(pos.y - r, pos.y + r) {
                        for (m_id, _) in map.entities_on_pos((x, y)) {
                            let monster = ID(m_id);
                            if ecm.has_entity(monster) && ecm.has_ai(monster) {
                                let ai = ecm.get_ai(monster);
                                ecm.set_ai(monster,
                                           AI{state: components::ai::Aggressive,
                                              .. ai});
                            }
                        }
                    }
                }
                Destination{x: player_pos.x, y: player_pos.y}
            }
            components::ai::Idle => {
                match random_neighbouring_position(rng, pos, map) {
                    (x, y) => Destination{x: x, y: y}
                }
            }
        }
    }

    pub fn system(e: ID,
                  ecm: &mut ComponentManager,
                  res: &mut Resources) {
        ensure_components!(ecm, e, AI, Position);
        ensure_components!(ecm, res.player_id, Position);
        if res.side != Computer {return}
        let player_pos = ecm.get_position(res.player_id);
        let pos = ecm.get_position(e);
        let dest = if entity_blocked(pos, &res.map) {
            println!("Found a blocked entity: {}", *e);
            Destination{x: pos.x, y: pos.y}
        } else {
            match ecm.get_ai(e).behaviour {
                components::ai::Individual => {
                    individual_behaviour(e, ecm, &mut res.rng, &mut res.map, player_pos)
                }
                components::ai::Pack => {
                    hunting_pack_behaviour(e, ecm, &mut res.rng, &mut res.map, player_pos)
                }
            }
        };
        ecm.set_destination(e, dest);
    }

}

pub mod panic {
    use components::*;
    use super::ai;
    use super::super::Resources;

    pub fn system(e: ID,
                  ecm: &mut ComponentManager,
                  res: &mut Resources) {
        ensure_components!(ecm, e, Panicking, Position, Destination);
        let pos = ecm.get_position(e);
        match ai::random_neighbouring_position(&mut res.rng, pos, &mut res.map) {
            (x, y) => ecm.set_destination(e, Destination{x: x, y: y}),
        }
    }
}

pub mod stun {
    use components::*;
    use super::super::Resources;

    pub fn system(e: ID,
                  ecm: &mut ComponentManager,
                  _res: &mut Resources) {
        ensure_components!(ecm, e, Stunned, Position, Destination);
        let Position{x, y} = ecm.get_position(e);
        ecm.set_destination(e, Destination{x: x, y: y});
    }
}

pub mod dose {
    use std::num;
    use components::*;
    use super::ai;
    use super::super::Resources;

    pub fn system(e: ID,
                  ecm: &mut ComponentManager,
                  res: &mut Resources) {
        ensure_components!(ecm, e, Addiction, Attributes, Position, Destination);
        let will = ecm.get_attributes(e).will;
        let pos = ecm.get_position(e);
        let search_radius = 3;  // max irresistibility for a dose is curretnly 3
        let mut doses: ~[ID] = ~[];
        for x in range(pos.x - search_radius, pos.x + search_radius) {
            for y in range(pos.y - search_radius, pos.y + search_radius) {
                for (dose_id, _) in res.map.entities_on_pos((x, y)) {
                    let dose = ID(dose_id);
                    if !ecm.has_entity(dose) {
                        fail2!("dose system: dose {} on pos {}, {} not in ecm.",
                               dose_id, x, y);
                    }
                    if !ecm.has_dose(dose) {loop};
                    let dose_pos = ecm.get_position(dose);
                    let path_to_dose = res.map.find_path((pos.x, pos.y), (dose_pos.x, dose_pos.y));
                    let resist_radius = num::max(ecm.get_dose(dose).resist_radius - will, 0);
                    let is_irresistible = match path_to_dose {
                        Some(p) => p.len() <= resist_radius,
                        None => false,
                    };
                    if is_irresistible {
                        doses.push(dose);
                    }
                }
            }
        }
        let nearest_dose = do doses.iter().min_by |&dose| {
            ai::distance(&ecm.get_position(*dose), &pos)
        };
        match nearest_dose {
            Some(&dose) => {
                let Position{x, y} = ecm.get_position(dose);
                // We walk the path here to make sure we only move one step at a
                // time.
                match res.map.find_path((pos.x, pos.y), (x, y)) {
                    Some(ref mut path) => {
                        let resist_radius = num::max(ecm.get_dose(dose).resist_radius - will, 0);
                        if path.len() <= resist_radius {
                            match path.walk() {
                                Some((x, y)) => {
                                    ecm.set_destination(e, Destination{x: x, y: y});
                                }
                                None => unreachable!(),
                            }
                        }
                    }
                    None => {}
                }
            }
            None => {}
        }
    }
}

pub mod movement {
    use components::*;
    use map::{Walkable, Solid};
    use super::ai;
    use super::super::Resources;

    pub fn system(e: ID,
                  ecm: &mut ComponentManager,
                  res: &mut Resources) {
        ensure_components!(ecm, e, Position, Destination, Turn);
        let turn = ecm.get_turn(e);
        if turn.ap <= 0 {return}

        let pos = ecm.get_position(e);
        let dest = ecm.get_destination(e);
        if (pos.x, pos.y) == (dest.x, dest.y) {
            // Wait (spends an AP but do nothing)
            println!("Entity {} waits.", *e);
            ecm.set_turn(e, turn.spend_ap(1));
            ecm.remove_destination(e);
        } else if ai::distance(&pos, &Position{x: dest.x, y: dest.y}) == 1 {
            if res.map.is_walkable((dest.x, dest.y))  {  // Move to the cell
                ecm.set_turn(e, turn.spend_ap(1));
                { // Update both the entity position component and the map:
                    res.map.move_entity(*e, (pos.x, pos.y), (dest.x, dest.y));
                    ecm.set_position(e, Position{x: dest.x, y: dest.y});
                }
                ecm.remove_destination(e);
            } else {  // Bump into the blocked entity
                // TODO: assert there's only one solid entity on pos [x, y]
                for (bumpee, walkable) in res.map.entities_on_pos((dest.x, dest.y)) {
                    assert!(bumpee != *e);
                    match walkable {
                        Walkable => loop,
                        Solid => {
                            println!("Entity {} bumped into {} at: ({}, {})",
                                     *e, bumpee, dest.x, dest.y);
                            ecm.set_bump(e, Bump(ID(bumpee)));
                            ecm.remove_destination(e);
                            break;
                        }
                    }
                }
            }
        } else {  // Farther away than 1 space. Need to use path finding
            match res.map.find_path((pos.x, pos.y), (dest.x, dest.y)) {
                Some(ref mut path) => {
                    assert!(path.len() > 1,
                            "The path shouldn't be trivial. We already handled that.");
                    match path.walk() {
                        Some((x, y)) => {
                            let new_pos = Position{x: x, y: y};
                            assert!(ai::distance(&pos, &new_pos) == 1,
                                    "The step should be right next to the curret pos.");
                            ecm.set_turn(e, turn.spend_ap(1));
                            { // Update both the entity position component and the map:
                                res.map.move_entity(*e, (pos.x, pos.y), (x, y));
                                ecm.set_position(e, new_pos);
                            }
                        }
                        // "The path exists but can't be walked?!"
                        None => unreachable!(),
                    }
                }
                None => {
                    println!("Entity {} cannot find a path so it waits.", *e);
                    ecm.set_turn(e, turn.spend_ap(1));
                    ecm.remove_destination(e);
                }
            }
        }
    }
}

pub mod interaction {
    use components::*;
    use super::combat;
    use super::super::Resources;

    pub fn system(e: ID,
                  ecm: &mut ComponentManager,
                  res: &mut Resources) {
        // Only humans can use stuff for now:
        ensure_components!(ecm, e, AcceptsUserInput, Position);
        let pos = match ecm.get_position(e) {Position{x, y} => (x, y)};
        for (entity_map_id, _walkability) in res.map.entities_on_pos(pos) {
            let inter = ID(entity_map_id);
            if e == inter { loop }  // Entity cannot interact with itself
            if !ecm.has_entity(inter) {loop}
            let is_interactive = ecm.has_attribute_modifier(inter) || ecm.has_explosion_effect(inter);
            if !is_interactive {loop}
            let is_dose = ecm.has_dose(inter);
            if ecm.has_attribute_modifier(inter) {
                let tolerance = if is_dose && ecm.has_addiction(e) {
                    ecm.get_addiction(e).tolerance
                } else {
                    0
                };
                if ecm.has_attributes(e) {
                    let attrs = ecm.get_attributes(e);
                    let modifier = ecm.get_attribute_modifier(inter);
                    ecm.set_attributes(e, Attributes{
                            state_of_mind: attrs.state_of_mind + modifier.state_of_mind - tolerance,
                            will: attrs.will + modifier.will,
                        });
                }
            }
            if is_dose {
                if ecm.has_addiction(e) {
                    let addiction = ecm.get_addiction(e);
                    let dose = ecm.get_dose(inter);
                    ecm.set_addiction(e, Addiction{
                            tolerance: addiction.tolerance + dose.tolerance_modifier,
                            .. addiction});
                }
            }
            if ecm.has_explosion_effect(inter) {
                let radius = ecm.get_explosion_effect(inter).radius;
                let (px, py) = pos;
                for x in range(px - radius, px + radius) {
                    for y in range(py - radius, py + radius) {
                        for (m_id, _) in res.map.entities_on_pos((x, y)) {
                            let monster = ID(m_id);
                            if ecm.has_entity(monster) && ecm.has_ai(monster) {
                                combat::kill_entity(monster, ecm, &mut res.map);
                            }
                        }
                    }
                }
            }
            ecm.remove_position(inter);
            res.map.remove_entity(*inter, pos);
        }
    }
}

pub mod bump {
    use components::*;
    use super::super::Resources;

    pub fn system(e: ID,
                  ecm: &mut ComponentManager,
                  _res: &mut Resources) {
        ensure_components!(ecm, e, Bump)
        let bumpee = *ecm.get_bump(e);
        ecm.remove_bump(e);
        if !ecm.has_entity(bumpee) {return}
        let different_sides = (ecm.has_turn(bumpee) && ecm.has_turn(e)
                               && ecm.get_turn(bumpee).side != ecm.get_turn(e).side);
        if different_sides {
            println!("Entity {} attacks {}.", *e, *bumpee);
            ecm.set_attack_target(e, AttackTarget(bumpee));
        } else {
            println!("Entity {} hits the wall.", *e);
        }
    }
}

pub mod combat {
    use components::*;
    use map::{Map};
    use super::super::Resources;

    pub fn kill_entity(e: ID,
                       ecm: &mut ComponentManager,
                       map: &mut Map) {
        if !ecm.has_entity(e) {return}
        ecm.remove_ai(e);
        if ecm.has_position(e) {
            let Position{x, y} = ecm.get_position(e);
            ecm.remove_position(e);
            map.remove_entity(*e, (x, y));
        }
        ecm.remove_accepts_user_input(e);
        ecm.remove_turn(e);
    }

    pub fn system(e: ID,
                  ecm: &mut ComponentManager,
                  res: &mut Resources) {
        ensure_components!(ecm, e, AttackTarget, AttackType, Turn);
        let free_aps = ecm.get_turn(e).ap;
        let target = *ecm.get_attack_target(e);
        ecm.remove_attack_target(e);
        let attack_successful = ecm.has_entity(target) && free_aps > 0;
        if !attack_successful {return}
        // attacker spends an AP
        let turn = ecm.get_turn(e);
        ecm.set_turn(e, turn.spend_ap(1));
        match ecm.get_attack_type(e) {
            Kill => {
                println!("Entity {} was killed by {}", *target, *e);
                kill_entity(target, ecm, &mut res.map);
                let target_is_anxiety = (ecm.has_monster(target) &&
                                         ecm.get_monster(target).kind == Anxiety);
                if target_is_anxiety && ecm.has_anxiety_kill_counter(e) {
                    let counter = ecm.get_anxiety_kill_counter(e);
                    ecm.set_anxiety_kill_counter(e, AnxietyKillCounter{
                            count: counter.count + 1,
                            .. counter
                        });
                }
            }
            Stun{duration} => {
                println!("Entity {} was stunned by {}", *target, *e);
                // An attacker with stun disappears after delivering the blow
                kill_entity(e, ecm, &mut res.map);
                let stunned = if ecm.has_stunned(target) {
                    let prev = ecm.get_stunned(target);
                    Stunned{duration: prev.duration + duration, .. prev}
                } else {
                    Stunned{turn: res.turn, duration: duration}
                };
                ecm.set_stunned(target, stunned);
            }
            Panic{duration} => {
                println!("Entity {} panics because of {}", *target, *e);
                // An attacker with stun disappears after delivering the blow
                kill_entity(e, ecm, &mut res.map);
                let panicking = if ecm.has_panicking(target) {
                    let prev = ecm.get_panicking(target);
                    Panicking{duration: prev.duration + duration, .. prev}
                } else {
                    Panicking{turn: res.turn, duration: duration}
                };
                ecm.set_panicking(target, panicking);
            }
            ModifyAttributes => {
                if !ecm.has_attribute_modifier(e) {
                    fail!("The attacker must have attribute_modifier");
                }
                let modifier = ecm.get_attribute_modifier(e);
                if ecm.has_attributes(target) {
                    let attrs = ecm.get_attributes(target);
                    ecm.set_attributes(target, Attributes{
                            state_of_mind: attrs.state_of_mind + modifier.state_of_mind,
                            will: attrs.will + modifier.will})
                }
            }
        }
    }
}


mod effect_duration {
    use components::*;
    use super::super::Resources;

    pub fn system(entity: ID,
                  ecm: &mut ComponentManager,
                  res: &mut Resources) {
        if !ecm.has_entity(entity) {return}

        if ecm.has_stunned(entity) {
            let stunned = ecm.get_stunned(entity);
            if stunned.remaining(res.turn) == 0 {
                ecm.remove_stunned(entity);
            }
        }
        if ecm.has_panicking(entity) {
            let panicking = ecm.get_panicking(entity);
            if panicking.remaining(res.turn) == 0 {
                ecm.remove_panicking(entity);
            }
        }
    }
}

mod addiction {
    use components::*;
    use super::combat;
    use super::super::Resources;

    pub fn system(e: ID,
                  ecm: &mut ComponentManager,
                  res: &mut Resources) {
        ensure_components!(ecm, e, Addiction, Attributes);
        let addiction = ecm.get_addiction(e);
        let attr = ecm.get_attributes(e);
        if res.turn > addiction.last_turn {
            ecm.set_attributes(e, Attributes{
                    state_of_mind: attr.state_of_mind - addiction.drop_per_turn,
                    .. attr
                });
            ecm.set_addiction(e, Addiction{last_turn: res.turn, .. addiction});
        };
        let som = ecm.get_attributes(e).state_of_mind;
        if som <= 0 || som >= 100 {
            combat::kill_entity(e, ecm, &mut res.map);
        }
    }
}

mod will {
    use components::*;
    use super::combat;
    use super::super::Resources;

    pub fn system(e: ID,
                  ecm: &mut ComponentManager,
                  res: &mut Resources) {
        ensure_components!(ecm, e, Attributes);
        let attrs = ecm.get_attributes(e);

        if ecm.has_anxiety_kill_counter(e) {
            let kc = ecm.get_anxiety_kill_counter(e);
            if kc.count >= kc.threshold {
                ecm.set_attributes(e,
                                   Attributes{will: attrs.will + 1, .. attrs});
                ecm.set_anxiety_kill_counter(e,
                                             AnxietyKillCounter{
                        count: kc.threshold - kc.count,
                        .. kc
                    });
            }
        }
        if ecm.get_attributes(e).will <= 0 {
            combat::kill_entity(e, ecm, &mut res.map);
        }
    }
}

pub mod tile {
    use components::{ComponentManager, ID, Position, Tile};
    use engine::{Color, Display};

    pub fn system(entity: ID, ecm: &ComponentManager, display: &mut Display) {
        if !ecm.has_entity(entity) {return}
        if !ecm.has_position(entity) { return }
        if !ecm.has_tile(entity) { return }

        let Position{x, y} = ecm.get_position(entity);
        let Tile{level, glyph, color} = ecm.get_tile(entity);
        display.draw_char(level, x, y, glyph, color, Color::new(20, 20, 20));
    }
}

pub mod turn {
    use components;
    use components::*;
    use super::super::Resources;

    impl components::Side {
        fn next(&self) -> Side {
            match *self {
                Player => Computer,
                Computer => Player,
            }
        }

        fn is_last(&self) -> bool {
            *self == Computer
        }
    }

    pub fn system(ecm: &mut ComponentManager,
                  res: &mut Resources) {
        let switch_sides = ecm.iter().all(|e| {
                match ecm.has_turn(e) {
                    true => {
                        let turn = ecm.get_turn(e);
                        (res.side != turn.side) || (turn.ap == 0)
                    },
                    false => true,
                }
            });
        if switch_sides {
            if res.side.is_last() {
                res.turn += 1;
            }
            res.side = res.side.next();
            for e in ecm.iter() {
                match ecm.has_turn(e) {
                    true => {
                        let turn = ecm.get_turn(e);
                        if turn.side == res.side {
                            ecm.set_turn(e, Turn{
                                    ap: turn.max_ap,
                                    .. turn});
                        }
                    },
                    false => (),
                }
            }
        }
    }
}

pub mod player_dead {
    use components::{ComponentManager, ID};
    use super::super::Resources;

    pub fn system(id: ID,
                  ecm: &mut ComponentManager,
                  res: &mut Resources) {
        let player = res.player_id;
        if !ecm.has_entity(player) {
            fail!("Could not find the Player entity (id: %?)", res.player_id)
        }
        let player_dead = !ecm.has_position(player) || !ecm.has_turn(player);
        if player_dead {
            match ecm.has_ai(id) {
                true => ecm.remove_ai(id),
                false => (),
            }
        }
    }
}

pub mod gui {
    use engine::{Display, Color};
    use components::*;
    use super::super::Resources;

    pub fn system(ecm: &ComponentManager,
                  res: &mut Resources,
                  display: &mut Display) {
        let (_width, height) = display.size();
        let player = res.player_id;
        ensure_components!(ecm, player, Attributes);
        let attrs = ecm.get_attributes(player);
        let dead = match ecm.has_position(player) {
            true => ~"",
            false => ~"dead ",
        };
        let stunned = match ecm.has_stunned(player) {
            true => format!("stunned({}) ", ecm.get_stunned(player).remaining(res.turn)),
            false => ~"",
        };
        let panicking = match ecm.has_panicking(player) {
            true => format!("panic({}) ", ecm.get_panicking(player).remaining(res.turn)),
            false => ~"",
        };
        let effects = format!("{}{}{}", dead, stunned, panicking);
        let status_bar = format!("Intoxication: {},  Will: {}, Effects: {}",
                                 attrs.state_of_mind,
                                 attrs.will,
                                 effects);
        display.write_text(status_bar,
                           0, height - 1,
                           Color::new(255, 255, 255), Color::new(0, 0, 0));
    }
}
