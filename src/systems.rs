use components::*;
use engine::{Display, Color};
use extra::container::Deque;
use extra::ringbuf::RingBuf;
use map;
use super::CommandLogger;
use entity_manager::{EntityManager, ID};


#[deriving(Rand, ToStr)]
pub enum Command {
    N, E, S, W, NE, NW, SE, SW,
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

pub fn turn_tick_counter_system(id: ID, ecm: &mut EntityManager<GameObject>, current_side: Side) {
    match ecm.get_mut_ref(id) {
        Some(entity) => {
            entity.turn.mutate(|t| if t.side == current_side {
                    Turn{spent_this_tick: 0, .. t}
                } else {
                    t
                });
        }
        None => {}
    }
}

pub fn input_system(id: ID, ecm: &mut EntityManager<GameObject>, commands: &mut RingBuf<Command>,
                    logger: CommandLogger, current_side: Side) {
    if ecm.get_ref(id).is_none() { return }
    let entity = ecm.get_mut_ref(id).unwrap();

    if entity.accepts_user_input.is_none() { return }
    if entity.position.is_none() { return }
    match current_side {
        Player => (),
        _ => return,
    }

    let pos = entity.position.get_ref();
    match commands.pop_front() {
        Some(command) => {
            logger.log(command);
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
            entity.destination = Some(dest);
        },
        None => (),
    }
}


pub mod ai {
    use std::rand::Rng;
    use entity_manager::{ID, EntityManager};
    use components::*;
    use components;
    use map::Map;
    use std::num::{abs, max};


    fn distance(p1: &Position, p2: &Position) -> int {
        max(abs(p1.x - p2.x), abs(p1.y - p2.y))
    }

    fn random_neighbouring_destination<T: Rng>(rng: &mut T,
                                               pos: Position,
                                               map: &Map) -> Destination {
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
            Destination{x: pos.x, y: pos.y}  // Nowhere to go
        } else {
            match rng.choose(walkables) {
                (x, y) => Destination{x: x, y: y}
            }
        }
    }

    fn individual_behaviour<T: Rng>(id: ID,
                                    ecm: &mut EntityManager<GameObject>,
                                    rng: &mut T,
                                    map: &Map,
                                    player_pos: Position) -> Destination {
        let e = ecm.get_mut_ref(id).unwrap();
        let pos = e.position.unwrap();
        let player_distance = distance(&pos, &player_pos);
        match player_distance {
            dist if dist < 5 => e.ai.get_mut_ref().state = components::ai::Aggressive,
            dist if dist > 8 => e.ai.get_mut_ref().state = components::ai::Idle,
            _ => {}
        }
        match e.ai.get_ref().state {
            components::ai::Aggressive => {
                Destination{x: player_pos.x, y: player_pos.y}
            }
            components::ai::Idle => {
                random_neighbouring_destination(rng, pos, map)
            }
        }
    }

    fn hunting_pack_behaviour<T: Rng>(id: ID,
                                      ecm: &mut EntityManager<GameObject>,
                                      rng: &mut T,
                                      map: &Map,
                                      player_pos: Position) -> Destination {
        let pos = ecm.get_ref(id).unwrap().position.unwrap();
        let state = match ecm.get_mut_ref(id) {
            Some(e) => {
                let player_distance = distance(&pos, &player_pos);
                if player_distance < 4 {
                    e.ai.get_mut_ref().state = components::ai::Aggressive
                }
                e.ai.get_ref().state
            }
            None => fail!("Unreachable: the entity must be available here"),
        };
        match state {
            components::ai::Aggressive => {
                let r = 8;
                for x in range(pos.x - r, pos.x + r) {
                    for y in range(pos.y - r, pos.y + r) {
                        for (m_id, _) in map.entities_on_pos((x, y)) {
                            match ecm.get_mut_ref(ID(m_id)) {
                                Some(m) => if m.ai.is_some() {
                                    m.ai.get_mut_ref().state = components::ai::Aggressive;
                                },
                                None => {}
                            }
                        }
                    }
                }
                Destination{x: player_pos.x, y: player_pos.y}
            }
            components::ai::Idle => {
                random_neighbouring_destination(rng, pos, map)
            }
        }
    }

    pub fn process<T: Rng>(id: ID, ecm: &mut EntityManager<GameObject>, rng: &mut T, map: &Map, current_side: Side, player_id: ID) {
        match ecm.get_ref(id) {
            Some(e) => {
                if e.ai.is_none() || e.position.is_none() { return }
            }
            None => { return }
        }
        match current_side {
            Computer => (),
            _ => return,
        }

        let player_pos = match ecm.get_ref(player_id) {
            Some(p) if p.position.is_some() => p.position.unwrap(),
            _ => { return }
        };
        let dest = match ecm.get_ref(id).unwrap().ai.unwrap().behaviour {
            components::ai::Individual => individual_behaviour(id, ecm, rng, map, player_pos),
            components::ai::Pack => hunting_pack_behaviour(id, ecm, rng, map, player_pos),
        };
        ecm.get_mut_ref(id).unwrap().destination = Some(dest);
    }

}


pub fn path_system(id: ID, ecm: &mut EntityManager<GameObject>, map: &mut map::Map) {
    if ecm.get_ref(id).is_none() { return }
    let entity = ecm.get_mut_ref(id).unwrap();

    if entity.position.is_none() { return }

    match entity.destination {
        Some(dest) => {
            let pos = entity.position.get_ref();
            entity.path = map.find_path((pos.x, pos.y), (dest.x, dest.y));
        },
        None => (),
    }
    entity.destination = None;
}

pub fn movement_system(id: ID, ecm: &mut EntityManager<GameObject>, map: &mut map::Map) {
    if ecm.get_ref(id).is_none() { return }
    let entity = ecm.get_mut_ref(id).unwrap();

    if entity.position.is_none() { return }
    if entity.path.is_none() { return }
    if entity.turn.is_none() { return }

    if entity.turn.get_ref().ap <= 0 { return }

    match (*entity.path.get_mut_ref()).walk() {
        Some((x, y)) => {
            if map.is_walkable((x, y)) {  // Move to the cell
                entity.spend_ap(1);
                // Update both the entity position component and the map:
                match *entity.position.get_ref() {
                    Position{x: oldx, y: oldy} => {
                        map.move_entity(*id, (oldx, oldy), (x, y));
                        entity.position = Some(Position{x: x, y: y});
                    }
                };
            } else {  // Bump into the blocked entity
                // TODO: assert there's only one solid entity on pos [x, y]
                for (bumpee, walkable) in map.entities_on_pos((x, y)) {
                    assert!(bumpee != *id);
                    match walkable {
                        map::Walkable => loop,
                        map::Solid => {
                            println!("Entity {} bumped into {} at: ({}, {})", *id, bumpee, x, y);
                            entity.bump = Some(Bump(ID(bumpee)));
                            break;
                        }
                    }
                }
            }
        },
        None => return,
    }
}


pub fn bump_system(entity_id: ID, ecm: &mut EntityManager<GameObject>) {
    let bumpee_id = {match ecm.get_ref(entity_id).unwrap().bump {Some(id) => *id, None => {return}}};
    let bumpee = {ecm.get_ref(bumpee_id).unwrap().turn};
    match ecm.get_mut_ref(entity_id) {
        Some(e) => {
            if bumpee.is_some() && e.turn.is_some() && bumpee.unwrap().side != e.turn.unwrap().side {
                println!("Entity {} attacks {}.", *entity_id, *bumpee_id)
                e.attack_target = Some(AttackTarget(bumpee_id));
            } else {
                println!("Entity {} hits the wall.", *entity_id);
            }
            e.bump = None;
        }
        _ => (),
    }
}

pub fn combat_system(id: ID, ecm: &mut EntityManager<GameObject>, map: &mut map::Map) {
    if ecm.get_ref(id).is_none() { return }
    if ecm.get_ref(id).unwrap().attack_target.is_none() { return }
    if ecm.get_ref(id).unwrap().attack_type.is_none() { return }
    let free_aps = match ecm.get_ref(id).unwrap().turn {
        Some(t) => t.ap,
        None => 0,
    };
    let target_id = match ecm.get_ref(id) {
        Some(e) => match e.attack_target {
            Some(attack_component) => *attack_component,
            None => return,
        },
        None => { return }
    };
    let attack_successful = ecm.get_ref(target_id).is_some() && free_aps > 0;
    if attack_successful {
        // attacker spends an AP
        match ecm.get_mut_ref(id) {
            Some(attacker) => {
                attacker.spend_ap(1);
                attacker.attack_target = None;
            }
            None => {}
        }
        let attack_type = ecm.get_ref(id).unwrap().attack_type.unwrap();
        let target = ecm.get_mut_ref(target_id).unwrap();
        match attack_type {
            Kill => {
                println!("Entity {} was killed by {}", *target_id, *id);
                target.ai = None;
                match target.position {
                    Some(Position{x, y}) => {
                        target.position = None;
                        map.remove_entity(*target_id, (x, y));
                    }
                    None => {}
                }
                target.accepts_user_input = None;
                target.turn = None;
            }
            Stun{duration} => {
                println!("Entity {} was stunned by {}", *target_id, *id);
                target.stunned.mutate_default(
                    Stunned{duration: duration},
                    |existing| Stunned{duration: existing.duration + duration});
            }
            Panic{duration} => {
                println!("Entity {} panics because of {}", *target_id, *id);
                target.panicking.mutate_default(
                    Panicking{duration: duration},
                    |existing| Panicking{duration: existing.duration + duration});
            }
            ModifyAttributes{state_of_mind, will} => {
                target.attributes.mutate(
                    |attrs| Attributes{
                        state_of_mind: attrs.state_of_mind + state_of_mind,
                        will: attrs.will + will});
            }
        }
    }
}

pub fn tile_system(id: ID, ecm: &EntityManager<GameObject>, display: &mut Display) {
    if ecm.get_ref(id).is_none() { return }
    let entity = ecm.get_ref(id).unwrap();

    if entity.position.is_none() { return }
    if entity.tile.is_none() { return }

    let &Position{x, y} = entity.position.get_ref();
    let &Tile{level, glyph, color} = entity.tile.get_ref();
    display.draw_char(level, x as uint, y as uint, glyph, color, Color(20, 20, 20));
}

pub fn idle_ai_system(id: ID, ecm: &mut EntityManager<GameObject>, current_side: Side) {
    if ecm.get_ref(id).is_none() { return }
    let entity = ecm.get_mut_ref(id).unwrap();

    if entity.turn.is_none() { return }
    if entity.ai.is_none() { return }
    if current_side != Computer { return }

    let turn = *entity.turn.get_ref();
    let is_idle = (turn.side == current_side) && turn.spent_this_tick == 0;
    if is_idle && turn.ap > 0 { entity.spend_ap(1) };
}


pub mod turn_system {
    use components;
    use components::*;
    use entity_manager::{EntityManager};

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

    pub fn run(entities: &mut EntityManager<GameObject>,
               current_side: &mut Side,
               current_turn: &mut int) {
        let switch_sides = entities.iter().all(|(e, _id)| {
                match e.turn {
                    Some(turn) => {
                        (*current_side != turn.side) || (turn.ap == 0)
                    },
                    None => true,
                }
            });
        if switch_sides {
            if current_side.is_last() {
                *current_turn += 1;
            }
            *current_side = current_side.next();
            for (e, _id) in entities.mut_iter() {
                match e.turn {
                    Some(turn) => {
                        if turn.side == *current_side {
                            e.turn = Some(Turn{
                                    ap: turn.max_ap,
                                    .. turn});
                        }
                    },
                    None => (),
                }
            }
        }
    }


}


pub fn player_dead_system(id: ID, ecm: &mut EntityManager<GameObject>, player_id: ID) {
    let player_dead = match ecm.get_ref(player_id) {
        Some(player) => {
            player.position.is_none() || player.turn.is_none()
        }
        None => fail!("Could not find the Player entity (id: %?)", player_id),
    };
    if player_dead {
        match ecm.get_mut_ref(id) {
            Some(e) => e.ai = None,
            None => (),
        }
    }
}

pub mod gui {
    use engine::{Display, Color};
    use components::*;
    use entity_manager::{EntityManager, ID};

    pub fn process(ecm: &EntityManager<GameObject>,
                   display: &mut Display,
                   player_id: ID) {
        let (_width, height) = display.size();
        let attrs = ecm.get_ref(player_id).unwrap().attributes.unwrap();
        let dead = match ecm.get_ref(player_id).unwrap().position.is_none() {
            true => ~"dead ",
            false => ~"",
        };
        let stunned = match ecm.get_ref(player_id).unwrap().stunned {
            Some(Stunned{duration}) => format!("stunned({}) ", duration),
            None => ~"",
        };
        let panicking = match ecm.get_ref(player_id).unwrap().panicking {
            Some(Panicking{duration}) => format!("panic({}) ", duration),
            None => ~"",
        };
        let effects = format!("{}{}{}", dead, stunned, panicking);
        let status_bar = format!("Intoxication: {},  Will: {}, Effects: {}",
                                 attrs.state_of_mind,
                                 attrs.will,
                                 effects);
        display.write_text(status_bar,
                           0, height - 1,
                           Color(255, 255, 255), Color(0, 0, 0));
    }
}
