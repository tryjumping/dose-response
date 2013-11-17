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
                    for monster in ecm.entities_on_pos(Position{x: x, y: y}) {
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
