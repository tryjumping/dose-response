use std::iter::range_inclusive;
use rand::Rng;

use ecm::{ComponentManager, ECM, Entity};
use components::ai;
use components::{AI, Destination, Position, Side, Computer};
use systems::movement::is_walkable;
use util::distance;


pub fn random_neighbouring_position<T: Rng>(rng: &mut T,
                                            pos: Position,
                                            ecm: &ECM,
                                            map_size: (int, int))
                                            -> (int, int) {
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
    let mut walkables: Vec<(int, int)> = Vec::new();
    for &p in neighbors.iter() {
        let pos = match p { (x, y) => Position{x: x, y: y} };
        if is_walkable(pos, ecm, map_size) { walkables.push(p) }
    }
    match rng.choose(walkables.slice(0, walkables.len())) {
        Some(&random_pos) => random_pos,
        None => (pos.x, pos.y)  // Nowhere to go
    }
}

pub fn entity_blocked(pos: Position, ecm: &ECM, map_size: (int, int))
                      -> bool {
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
    !neighbors.iter().any(|&neighbor_pos| {
        let pos = match neighbor_pos { (x, y) => Position{x: x, y: y}};
        is_walkable(pos, ecm, map_size)
    })
}

fn individual_behaviour<T: Rng>(e: Entity,
                                ecm: &mut ECM,
                                rng: &mut T,
                                map_size: (int, int),
                                player_pos: Position) -> Destination {
    let pos = ecm.get::<Position>(e);
    let player_distance = distance(&pos, &player_pos);
    let ai: AI = ecm.get(e);
    match player_distance {
        dist if dist < 5 => {
            ecm.set(e, AI{state: ai::Aggressive, .. ai});
        }
        dist if dist > 8 => {
            ecm.set(e, AI{state: ai::Idle, .. ai});
        }
        _ => {}
    }
    match ecm.get::<AI>(e).state {
        ai::Aggressive => {
            Destination{x: player_pos.x, y: player_pos.y}
        }
        ai::Idle => {
            match random_neighbouring_position(rng, pos, ecm, map_size) {
                (x, y) => Destination{x: x, y: y}
            }
        }
    }
}

fn hunting_pack_behaviour<T: Rng>(e: Entity,
                                  ecm: &mut ECM,
                                  rng: &mut T,
                                  map_size: (int, int),
                                  player_pos: Position) -> Destination {
    let pos = ecm.get::<Position>(e);
    let player_distance = distance(&pos, &player_pos);
    if player_distance < 4 {
        let ai: AI = ecm.get(e);
        ecm.set(e, AI{state: ai::Aggressive, .. ai});
    }
    match ecm.get::<AI>(e).state {
        ai::Aggressive => {
            let r = 8;
            for x in range_inclusive(pos.x - r, pos.x + r) {
                for y in range_inclusive(pos.y - r, pos.y + r) {
                    for monster in ecm.entities_on_pos((x, y)) {
                        if ecm.has_entity(monster) && ecm.has::<AI>(monster) {
                            let ai: AI = ecm.get(monster);
                            ecm.set(monster,
                                       AI{state: ai::Aggressive,
                                          .. ai});
                        }
                    }
                }
            }
            Destination{x: player_pos.x, y: player_pos.y}
        }
        ai::Idle => {
            match random_neighbouring_position(rng, pos, ecm, map_size) {
                (x, y) => Destination{x: x, y: y}
            }
        }
    }
}


define_system! {
    name: AISystem;
    components(AI, Position);
    resources(ecm: ECM, player: Entity, side: Side, world_size: (int, int), rng: ::rand::IsaacRng);
    fn process_entity(&mut self, dt_ms: uint, e: Entity) {
        let mut ecm = &mut *self.ecm();
        if !ecm.has::<Position>(*self.player()) { return }
        if *self.side() != Computer { return }

        let world_size = *self.world_size();
        let player_pos = ecm.get::<Position>(*self.player());
        let pos = ecm.get::<Position>(e);
        let mut rng = &mut *self.rng();
        let dest = if entity_blocked(pos, ecm, world_size) {
            println!("Found a blocked entity: {:?}", e);
            Destination{x: pos.x, y: pos.y}
        } else {
            match ecm.get::<AI>(e).behaviour {
                ai::Individual => {
                    individual_behaviour(e, ecm, rng, world_size, player_pos)
                }
                ai::Pack => {
                    hunting_pack_behaviour(e, ecm, rng, world_size, player_pos)
                }
            }
        };
        ecm.set(e, dest);
    }
}
