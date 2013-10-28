pub mod turn_tick_counter {
    use components::{ComponentManager, ID, Turn};
    use super::super::Resources;

    pub fn system(id: ID,
                  ecm: &mut ComponentManager,
                  res: &mut Resources) {
        if !ecm.has_turn(id) {return}
        let turn = ecm.get_turn(id);
        if turn.side == res.side {
            ecm.set_turn(id, Turn{spent_this_tick: 0, .. turn});
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

//     pub fn system(id: ID,
//                   ecm: &mut EntityManager<Entity>,
//                   res: &mut Resources) {
//         if ecm.get_ref(id).is_none() { return }
//         let entity = ecm.get_mut_ref(id).unwrap();

//         if entity.accepts_user_input.is_none() { return }
//         if entity.position.is_none() { return }
//         match res.side {
//             Player => (),
//             _ => return,
//         }

//         let pos = entity.position.get_ref();
//         match res.commands.pop_front() {
//             Some(command) => {
//                 res.command_logger.log(command);
//                 let dest = match command {
//                     N => Destination{x: pos.x, y: pos.y-1},
//                     S => Destination{x: pos.x, y: pos.y+1},
//                     W => Destination{x: pos.x-1, y: pos.y},
//                     E => Destination{x: pos.x+1, y: pos.y},

//                     NW => Destination{x: pos.x-1, y: pos.y-1},
//                     NE => Destination{x: pos.x+1, y: pos.y-1},
//                     SW => Destination{x: pos.x-1, y: pos.y+1},
//                     SE => Destination{x: pos.x+1, y: pos.y+1},
//                 };
//                 entity.destination = Some(dest);
//             },
//             None => (),
//         }
//     }
}


// pub mod leave_area {
//     use components::*;
//     use entity_manager::{ID, EntityManager};
//     use map::Map;
//     use world_gen;
//     use world;
//     use super::super::Resources;

//     pub fn system(id: ID,
//                   ecm: &mut EntityManager<Entity>,
//                   res: &mut Resources) {
//         if id != res.player_id {return}
//         if ecm.get_ref(res.player_id).is_none() {return}
//         let dest = match ecm.get_ref(res.player_id).unwrap().destination {
//             Some(dest) => dest,
//             None => {return}
//         };
//         let (x, y) = (dest.x as uint, dest.y as uint);
//         if x < 0 || y < 0 || x >= res.map.width || y >= res.map.height {
//             let mut player = ecm.take_out(res.player_id);
//             player.position = Some(Position{
//                     x: (res.map.width / 2) as int,
//                     y: (res.map.height / 2) as int,
//                 });
//             player.bump = None;
//             player.attack_target = None;
//             player.destination = None;
//             player.path = None;
//             ecm.clear();
//             res.map = Map::new(res.map.width, res.map.height);
//             let player_pos = player.position.unwrap();
//             let player_id = ecm.add(player);
//             res.player_id = player_id;
//             world::populate_world(ecm,
//                                   &mut res.map,
//                                   player_pos,
//                                   &mut res.rng,
//                                   world_gen::forrest);
//             // TODO: We don't want the curret tick to continue after we've messed with
//             // the game state. Signal the main loop to abort it early.
//         }
//     }
// }

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

//     pub fn random_neighbouring_position<T: Rng>(rng: &mut T,
//                                                 pos: Position,
//                                                 map: &Map) -> (int, int) {
//         let neighbors = [
//             (pos.x, pos.y-1),
//             (pos.x, pos.y+1),
//             (pos.x-1, pos.y),
//             (pos.x+1, pos.y),
//             (pos.x-1, pos.y-1),
//             (pos.x+1, pos.y-1),
//             (pos.x-1, pos.y+1),
//             (pos.x+1, pos.y+1),
//             ];
//         let mut walkables: ~[(int, int)] = ~[];
//         for &p in neighbors.iter() {
//             if map.is_walkable(p) { walkables.push(p) }
//         }
//         if walkables.is_empty() {
//             (pos.x, pos.y)  // Nowhere to go
//         } else {
//             rng.choose(walkables)
//         }
//     }

//     pub fn entity_blocked(pos: Position, map: &Map) -> bool {
//         let neighbors = [
//             (pos.x, pos.y-1),
//             (pos.x, pos.y+1),
//             (pos.x-1, pos.y),
//             (pos.x+1, pos.y),
//             (pos.x-1, pos.y-1),
//             (pos.x+1, pos.y-1),
//             (pos.x-1, pos.y+1),
//             (pos.x+1, pos.y+1),
//             ];
//         !do neighbors.iter().any |&neighbor_pos| {
//             map.is_walkable(neighbor_pos)
//         }
//     }

//     fn individual_behaviour<T: Rng>(id: ID,
//                                     ecm: &mut EntityManager<Entity>,
//                                     rng: &mut T,
//                                     map: &Map,
//                                     player_pos: Position) -> Destination {
//         let e = ecm.get_mut_ref(id).unwrap();
//         let pos = e.position.unwrap();
//         let player_distance = distance(&pos, &player_pos);
//         match player_distance {
//             dist if dist < 5 => e.ai.get_mut_ref().state = components::ai::Aggressive,
//             dist if dist > 8 => e.ai.get_mut_ref().state = components::ai::Idle,
//             _ => {}
//         }
//         match e.ai.get_ref().state {
//             components::ai::Aggressive => {
//                 Destination{x: player_pos.x, y: player_pos.y}
//             }
//             components::ai::Idle => {
//                 match random_neighbouring_position(rng, pos, map) {
//                     (x, y) => Destination{x: x, y: y}
//                 }
//             }
//         }
//     }

//     fn hunting_pack_behaviour<T: Rng>(id: ID,
//                                       ecm: &mut EntityManager<Entity>,
//                                       rng: &mut T,
//                                       map: &Map,
//                                       player_pos: Position) -> Destination {
//         let pos = ecm.get_ref(id).unwrap().position.unwrap();
//         let state = match ecm.get_mut_ref(id) {
//             Some(e) => {
//                 let player_distance = distance(&pos, &player_pos);
//                 if player_distance < 4 {
//                     e.ai.get_mut_ref().state = components::ai::Aggressive
//                 }
//                 e.ai.get_ref().state
//             }
//             None => fail!("Unreachable: the entity must be available here"),
//         };
//         match state {
//             components::ai::Aggressive => {
//                 let r = 8;
//                 for x in range(pos.x - r, pos.x + r) {
//                     for y in range(pos.y - r, pos.y + r) {
//                         for (m_id, _) in map.entities_on_pos((x, y)) {
//                             match ecm.get_mut_ref(ID(m_id)) {
//                                 Some(m) => if m.ai.is_some() {
//                                     m.ai.get_mut_ref().state = components::ai::Aggressive;
//                                 },
//                                 None => {}
//                             }
//                         }
//                     }
//                 }
//                 Destination{x: player_pos.x, y: player_pos.y}
//             }
//             components::ai::Idle => {
//                 match random_neighbouring_position(rng, pos, map) {
//                     (x, y) => Destination{x: x, y: y}
//                 }
//             }
//         }
//     }

//     pub fn system(id: ID,
//                   ecm: &mut EntityManager<Entity>,
//                   res: &mut Resources) {
//         match ecm.get_ref(id) {
//             Some(e) => {
//                 if e.ai.is_none() || e.position.is_none() { return }
//             }
//             None => { return }
//         }
//         match res.side {
//             Computer => (),
//             _ => return,
//         }

//         let player_pos = match ecm.get_ref(res.player_id) {
//             Some(p) if p.position.is_some() => p.position.unwrap(),
//             _ => { return }
//         };
//         let pos = ecm.get_ref(id).unwrap().position.unwrap();
//         let dest = if entity_blocked(pos, &res.map) {
//             println!("Found a blocked entity: {}", *id);
//             Destination{x: pos.x, y: pos.y}
//         } else {
//             match ecm.get_ref(id).unwrap().ai.unwrap().behaviour {
//                 components::ai::Individual => individual_behaviour(id, ecm, &mut res.rng, &mut res.map, player_pos),
//                 components::ai::Pack => hunting_pack_behaviour(id, ecm, &mut res.rng, &mut res.map, player_pos),
//             }
//         };
//         ecm.get_mut_ref(id).unwrap().destination = Some(dest);
//     }

}

// pub mod panic {
//     use components::{Destination, Entity};
//     use entity_manager::{EntityManager, ID};
//     use super::ai;
//     use super::super::Resources;

//     pub fn system(id: ID,
//                   ecm: &mut EntityManager<Entity>,
//                   res: &mut Resources) {
//         let e = match ecm.get_mut_ref(id) {
//             Some(e) => e,
//             None => {return}
//         };
//         if e.panicking.is_none() || e.destination.is_none() {return}
//         let pos = match e.position {
//             Some(pos) => pos,
//             None => unreachable!(),
//         };
//         match ai::random_neighbouring_position(&mut res.rng, pos, &mut res.map) {
//             (x, y) => e.destination = Some(Destination{x: x, y: y})
//         }
//     }
// }

// pub mod stun {
//     use components::{Destination, Entity};
//     use entity_manager::{EntityManager, ID};
//     use super::super::Resources;

//     pub fn system(id: ID,
//                   ecm: &mut EntityManager<Entity>,
//                   _res: &mut Resources) {
//         let e = match ecm.get_mut_ref(id) {
//             Some(e) => e,
//             None => {return}
//         };
//         if e.stunned.is_none() || e.destination.is_none() {return}
//         let pos = match e.position {
//             Some(pos) => pos,
//             None => unreachable!(),
//         };
//         e.destination = Some(Destination{x: pos.x, y: pos.y});
//     }
// }

// pub mod dose {
//     use std::num;
//     use components::*;
//     use entity_manager::{EntityManager, ID};
//     use super::ai;
//     use super::super::Resources;

//     pub fn system(id: ID,
//                   ecm: &mut EntityManager<Entity>,
//                   res: &mut Resources) {
//         if ecm.get_ref(id).is_none() {return}
//         if ecm.get_ref(id).unwrap().addiction.is_none() {return}
//         if ecm.get_ref(id).unwrap().attributes.is_none() {return}
//         if ecm.get_ref(id).unwrap().position.is_none() {return}
//         if ecm.get_ref(id).unwrap().destination.is_none() {
//             // Prevent the PC from running towards the dose without any input
//             // from the player:
//             ecm.get_mut_ref(id).unwrap().path = None;
//             return
//         }

//         let will = ecm.get_ref(id).unwrap().attributes.unwrap().will;
//         let search_radius = 3;  // max irresistibility for a dose is curretnly 3
//         let mut doses: ~[ID] = ~[];
//         let pos = ecm.get_ref(id).unwrap().position.unwrap();
//         for x in range(pos.x - search_radius, pos.x + search_radius) {
//             for y in range(pos.y - search_radius, pos.y + search_radius) {
//                 for (dose_id, _) in res.map.entities_on_pos((x, y)) {
//                     match ecm.get_ref(ID(dose_id)) {
//                         Some(dose) if dose.dose.is_some() => {
//                             let dose_pos = dose.position.unwrap();
//                             let path_to_dose = res.map.find_path((pos.x, pos.y), (dose_pos.x, dose_pos.y));
//                             let resist_radius = num::max(dose.dose.get_ref().resist_radius - will, 0);
//                             let is_irresistible = match path_to_dose {
//                                 Some(p) => p.len() <= resist_radius,
//                                 None => false,
//                             };
//                             if is_irresistible {
//                                 doses.push(ID(dose_id));
//                             }
//                         }
//                         _ => {}
//                     }
//                 }
//             }
//         }
//         let nearest_dose = do doses.iter().min_by |&dose| {
//             ai::distance(ecm.get_ref(*dose).unwrap().position.get_ref(), &pos)
//         };
//         match nearest_dose {
//             Some(&dose_id) => {
//                 let dose_pos = ecm.get_ref(dose_id).unwrap().position.unwrap();
//                 let dest = Destination{x: dose_pos.x, y: dose_pos.y};
//                 ecm.get_mut_ref(id).unwrap().destination = Some(dest);
//             }
//             None => {return}
//         }

//     }
// }

// pub mod path {
//     use components::{Entity};
//     use entity_manager::{EntityManager, ID};
//     use super::super::Resources;

//     pub fn system(id: ID,
//                   ecm: &mut EntityManager<Entity>,
//                   res: &mut Resources) {
//         if ecm.get_ref(id).is_none() { return }
//         let entity = ecm.get_mut_ref(id).unwrap();

//         if entity.position.is_none() { return }

//         match entity.destination {
//             Some(dest) => {
//                 let pos = entity.position.get_ref();
//                 entity.path = res.map.find_path((pos.x, pos.y), (dest.x, dest.y));
//                 if entity.path.is_none() {
//                     // if we can't find a path, make the entity wait by setting
//                     // the destination to the current position:
//                     entity.path = res.map.find_path((pos.x, pos.y), (pos.x, pos.y));
//                 }
//             },
//             None => (),
//         }
//         entity.destination = None;
//     }
// }


// pub mod movement {
//     use components::*;
//     use entity_manager::{EntityManager, ID};
//     use map::{Walkable, Solid};
//     use super::super::Resources;

//     pub fn system(id: ID,
//                   ecm: &mut EntityManager<Entity>,
//                   res: &mut Resources) {
//         if ecm.get_ref(id).is_none() { return }
//         let entity = ecm.get_mut_ref(id).unwrap();

//         if entity.position.is_none() { return }
//         if entity.path.is_none() { return }
//         if entity.turn.is_none() { return }

//         if entity.turn.get_ref().ap <= 0 { return }

//         let pos = entity.position.unwrap();
//         match (entity.path.get_mut_ref()).walk() {
//             Some(dest) => {
//                 let (x, y) = dest;
//                 if dest == (pos.x, pos.y) {  // Wait (spends an AP but do nothing)
//                     println!("Entity {} waits.", *id);
//                     entity.spend_ap(1);
//                 } else if res.map.is_walkable(dest) {  // Move to the cell
//                     entity.spend_ap(1);
//                     { // Update both the entity position component and the map:
//                         res.map.move_entity(*id, (pos.x, pos.y), dest);
//                         entity.position = Some(Position{x: x, y: y});
//                     }
//                 } else {  // Bump into the blocked entity
//                     // TODO: assert there's only one solid entity on pos [x, y]
//                     for (bumpee, walkable) in res.map.entities_on_pos(dest) {
//                         assert!(bumpee != *id);
//                         match walkable {
//                             Walkable => loop,
//                             Solid => {
//                                 println!("Entity {} bumped into {} at: ({}, {})", *id, bumpee, x, y);
//                                 entity.bump = Some(Bump(ID(bumpee)));
//                                 break;
//                             }
//                         }
//                     }
//                 }
//             }
//             None => {
//                 println!("Entity {} waits.", *id);
//                 entity.spend_ap(1);
//                 entity.path = None;
//             }
//         }
//     }
// }

// pub mod interaction {
//     use components::*;
//     use entity_manager::{EntityManager, ID};
//     use super::combat;
//     use super::super::Resources;

//     pub fn system(id: ID,
//                   ecm: &mut EntityManager<Entity>,
//                   res: &mut Resources) {
//         if ecm.get_ref(id).is_none() { return }
//         // Only humans can use stuff for now:
//         if ecm.get_ref(id).unwrap().accepts_user_input.is_none() { return }
//         let pos = match ecm.get_ref(id).unwrap().position {
//             Some(p) => (p.x, p.y),
//             None => return,
//         };
//         for (entity_map_id, _walkability) in res.map.entities_on_pos(pos) {
//             let interactive_id = ID(entity_map_id);
//             if id == interactive_id { loop }
//             match ecm.get_ref(interactive_id) {
//                 Some(i) => if i.attribute_modifier.is_some() || i.explosion_effect.is_some() {},
//                 _ => { loop }  // entity doesn't exist or isn't interactive
//             }
//             let is_dose = ecm.get_ref(interactive_id).unwrap().dose.is_some();
//             match ecm.get_ref(interactive_id).unwrap().attribute_modifier {
//                 Some(modifier) => {
//                     let tolerance = match ecm.get_ref(id).unwrap().addiction {
//                         Some(addiction) if is_dose => addiction.tolerance,
//                         _ => 0,
//                     };
//                     ecm.get_mut_ref(id).unwrap().attributes.mutate(
//                         |attrs| Attributes{
//                             state_of_mind: attrs.state_of_mind + modifier.state_of_mind - tolerance,
//                             will: attrs.will + modifier.will,
//                         });
//                 }
//                 None => {}
//             }
//             match ecm.get_ref(interactive_id).unwrap().dose {
//                 Some(dose) => {
//                     ecm.get_mut_ref(id).unwrap().addiction.mutate(
//                         |a| Addiction{
//                             tolerance: a.tolerance + dose.tolerance_modifier, .. a});
//                 }
//                 None => {}
//             }
//             match ecm.get_ref(interactive_id).unwrap().explosion_effect {
//                 Some(ExplosionEffect{radius}) => {
//                     let (px, py) = pos;
//                     for x in range(px - radius, px + radius) {
//                         for y in range(py - radius, py + radius) {
//                             for (m_id, _) in res.map.entities_on_pos((x, y)) {
//                                 let monster_id = ID(m_id);
//                                 if ecm.get_mut_ref(monster_id).unwrap().ai.is_some() {
//                                     combat::kill_entity(monster_id, ecm, &mut res.map);
//                                 }
//                             }
//                         }
//                     }
//                 }
//                 None => {}
//             }
//             ecm.get_mut_ref(interactive_id).unwrap().position = None;
//             res.map.remove_entity(*interactive_id, pos);
//         }
//     }
// }

// pub mod bump {
//     use components::{AttackTarget, Entity};
//     use entity_manager::{EntityManager, ID};
//     use super::super::Resources;

//     pub fn system(id: ID,
//                   ecm: &mut EntityManager<Entity>,
//                   _res: &mut Resources) {
//         let bumpee_id = match ecm.get_ref(id).unwrap().bump {
//             Some(id) => *id,
//             None => {return}
//         };
//         let bumpee = ecm.get_ref(bumpee_id).unwrap().turn;
//         match ecm.get_mut_ref(id) {
//             Some(e) => {
//                 if bumpee.is_some() && e.turn.is_some() && bumpee.unwrap().side != e.turn.unwrap().side {
//                     println!("Entity {} attacks {}.", *id, *bumpee_id);
//                     e.attack_target = Some(AttackTarget(bumpee_id));
//                 } else {
//                     println!("Entity {} hits the wall.", *id);
//                 }
//                 e.bump = None;
//             }
//             _ => (),
//         }
//     }
// }

// pub mod combat {
//     use components::*;
//     use entity_manager::{EntityManager, ID};
//     use map::{Map};
//     use super::super::Resources;

//     pub fn kill_entity(id: ID,
//                        ecm: &mut EntityManager<Entity>,
//                        map: &mut Map) {
//         match ecm.get_mut_ref(id) {
//             Some(e) => {
//                 e.ai = None;
//                 match e.position {
//                     Some(Position{x, y}) => {
//                         e.position = None;
//                         map.remove_entity(*id, (x, y));
//                     }
//                     None => {}
//                 }
//                 e.accepts_user_input = None;
//                 e.turn = None;
//             }
//             None => {}
//         }
//     }

//     pub fn system(id: ID,
//                   ecm: &mut EntityManager<Entity>,
//                   res: &mut Resources) {
//         if ecm.get_ref(id).is_none() { return }
//         if ecm.get_ref(id).unwrap().attack_target.is_none() { return }
//         if ecm.get_ref(id).unwrap().attack_type.is_none() { return }
//         let free_aps = match ecm.get_ref(id).unwrap().turn {
//             Some(t) => t.ap,
//             None => 0,
//         };
//         let target_id = match ecm.get_ref(id) {
//             Some(e) => match e.attack_target {
//                 Some(attack_component) => *attack_component,
//                 None => return,
//             },
//             None => { return }
//         };
//         let attack_successful = ecm.get_ref(target_id).is_some() && free_aps > 0;
//         if attack_successful {
//             // attacker spends an AP
//             match ecm.get_mut_ref(id) {
//                 Some(attacker) => {
//                     attacker.spend_ap(1);
//                     attacker.attack_target = None;
//                 }
//                 None => {}
//             }
//             let attack_type = ecm.get_ref(id).unwrap().attack_type.unwrap();
//             match attack_type {
//                 Kill => {
//                     println!("Entity {} was killed by {}", *target_id, *id);
//                     kill_entity(target_id, ecm, &mut res.map);
//                     let target_is_anxiety = match ecm.get_ref(target_id).unwrap().monster {
//                         Some(m) => m.kind == Anxiety,
//                         None => false,
//                     };
//                     match ecm.get_mut_ref(id) {
//                         Some(ref mut e) if target_is_anxiety && e.anxiety_kill_counter.is_some() => {
//                             do e.anxiety_kill_counter.mutate |counter| {
//                                 AnxietyKillCounter{
//                                     count: counter.count + 1,
//                                     .. counter
//                                 }
//                             };
//                         }
//                         _ => {}
//                     }
//                 }
//                 Stun{duration} => {
//                     println!("Entity {} was stunned by {}", *target_id, *id);
//                     kill_entity(id, ecm, &mut res.map);
//                     let target = ecm.get_mut_ref(target_id).unwrap();
//                     target.stunned.mutate_default(
//                         Stunned{turn: res.turn, duration: duration},
//                         |existing| Stunned{duration: existing.duration + duration, .. existing});
//                 }
//                 Panic{duration} => {
//                     println!("Entity {} panics because of {}", *target_id, *id);
//                     kill_entity(id, ecm, &mut res.map);
//                     let target = ecm.get_mut_ref(target_id).unwrap();
//                     target.panicking.mutate_default(
//                         Panicking{turn: res.turn, duration: duration},
//                         |existing| Panicking{duration: existing.duration + duration, .. existing});
//                 }
//                 ModifyAttributes => {
//                     match ecm.get_ref(id).unwrap().attribute_modifier {
//                         Some(modifier) => {
//                             let target = ecm.get_mut_ref(target_id).unwrap();
//                             target.attributes.mutate(
//                                 |attrs| Attributes{
//                                     state_of_mind: attrs.state_of_mind + modifier.state_of_mind,
//                                     will: attrs.will + modifier.will});

//                         }
//                         None => fail!("The attacker must have attribute_modifier"),
//                     }
//                 }
//             }
//         }
//     }
// }


// mod effect_duration {
//     use components::*;
//     use entity_manager::{EntityManager, ID};
//     use super::super::Resources;

//     pub fn system(id: ID,
//                   ecm: &mut EntityManager<Entity>,
//                   res: &mut Resources) {
//         match ecm.get_mut_ref(id) {
//             Some(e) => {
//                 e.stunned = do e.stunned.and_then |t| {
//                     if t.remaining(res.turn) == 0 {None} else {Some(t)}
//                 };
//                 e.panicking = do e.panicking.and_then |t| {
//                     if t.remaining(res.turn) == 0 {None} else {Some(t)}
//                 };
//             }
//             None => {}
//         }
//     }
// }

// mod addiction {
//     use components::*;
//     use entity_manager::{EntityManager, ID};
//     use super::combat;
//     use super::super::Resources;

//     pub fn system(id: ID,
//                   ecm: &mut EntityManager<Entity>,
//                   res: &mut Resources) {
//         match ecm.get_mut_ref(id) {
//             Some(ref mut e) if e.addiction.is_some() && e.attributes.is_some() => {
//                 let addiction = e.addiction.unwrap();
//                 if res.turn > addiction.last_turn {
//                     do e.attributes.mutate |attr| {
//                         Attributes{
//                             state_of_mind: attr.state_of_mind - addiction.drop_per_turn,
//                             .. attr
//                         }
//                     };
//                     do e.addiction.mutate |add| {
//                         Addiction{last_turn: res.turn, .. add}
//                     };
//                 }
//             }
//             _ => {return}
//         }
//         let som = ecm.get_ref(id).unwrap().attributes.unwrap().state_of_mind;
//         if som <= 0 || som >= 100 {
//             combat::kill_entity(id, ecm, &mut res.map);
//         }
//     }
// }

// mod will {
//     use components::*;
//     use entity_manager::{EntityManager, ID};
//     use super::combat;
//     use super::super::Resources;

//     pub fn system(id: ID,
//                   ecm: &mut EntityManager<Entity>,
//                   res: &mut Resources) {
//         if ecm.get_ref(id).is_none() { return }
//         if ecm.get_ref(id).unwrap().attributes.is_none() { return }

//         match ecm.get_mut_ref(id) {
//             Some(ref mut e) if e.anxiety_kill_counter.is_some() => {
//                 let kc = e.anxiety_kill_counter.unwrap();
//                 if kc.count >= kc.threshold {
//                     do e.attributes.mutate |attrs| {
//                         Attributes{will: attrs.will + 1, .. attrs}
//                     };
//                     do e.anxiety_kill_counter.mutate |counter| {
//                         AnxietyKillCounter{
//                             count: counter.threshold - counter.count,
//                             .. counter
//                         }
//                     };
//                 }
//             }
//             _ => {}
//         }

//         let attrs = ecm.get_ref(id).unwrap().attributes.unwrap();
//         if attrs.will <= 0 {
//             combat::kill_entity(id, ecm, &mut res.map);
//         }
//     }
// }

pub mod tile {
    use components::{ComponentManager, ID, Position, Tile};
    use engine::{Color, Display};

    pub fn system(entity: ID, ecm: &ComponentManager, display: &mut Display) {
        if !ecm.has_entity(entity) {return}
        if !ecm.has_position(entity) { return }
        if !ecm.has_tile(entity) { return }

        let Position{x, y} = ecm.get_position(entity);
        let Tile{level, glyph, color} = ecm.get_tile(entity);
        display.draw_char(level, x as uint, y as uint, glyph, color, Color(20, 20, 20));
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
        if !ecm.has_entity(player) {return}
        if !ecm.has_attributes(player) {return}

        let attrs = ecm.get_attributes(player);
        let dead = match ecm.has_position(player) {
            true => ~"dead ",
            false => ~"",
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
                           Color(255, 255, 255), Color(0, 0, 0));
    }
}
