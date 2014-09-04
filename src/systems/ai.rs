use std::iter::range_inclusive;
use std::rand::Rng;
use std::time::Duration;

use emhyr::{Components, Entity};
use components::ai;
use components::{AI, Destination, Position, Side, Computer, Turn};
use entity_util::{PositionCache, is_walkable};
use point;


pub fn random_neighbouring_position<T: Rng>(rng: &mut T,
                                            pos: Position,
                                            cache: &PositionCache,
                                            cs: &Components,
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
    let mut walkables: Vec<(int, int)> = vec![];
    for &p in neighbors.iter() {
        let pos = match p { (x, y) => Position{x: x, y: y} };
        if is_walkable(pos, cache, cs, map_size) { walkables.push(p) }
    }
    match rng.choose(walkables.slice(0, walkables.len())) {
        Some(&random_pos) => random_pos,
        None => (pos.x, pos.y)  // Nowhere to go
    }
}

pub fn entity_blocked(pos: Position, cache: &PositionCache, cs: &Components, map_size: (int, int))
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
        is_walkable(pos, cache, cs, map_size)
    })
}

fn individual_behaviour<T: Rng>(e: Entity,
                                cache: &PositionCache,
                                cs: &mut Components,
                                rng: &mut T,
                                map_size: (int, int),
                                player_pos: Position) -> Destination {
    let pos = cs.get::<Position>(e);
    let player_distance = point::tile_distance(pos, player_pos);
    let ai = cs.get::<AI>(e);
    match player_distance {
        dist if dist < 5 => {
            cs.set(AI{state: ai::Aggressive, .. ai}, e);
        }
        dist if dist > 8 => {
            cs.set(AI{state: ai::Idle, .. ai}, e);
        }
        _ => {}
    }
    match cs.get::<AI>(e).state {
        ai::Aggressive => {
            Destination{x: player_pos.x, y: player_pos.y}
        }
        ai::Idle => {
            match random_neighbouring_position(rng, pos, cache, cs, map_size) {
                (x, y) => Destination{x: x, y: y}
            }
        }
    }
}

fn hunting_pack_behaviour<T: Rng>(e: Entity,
                                  cache: &PositionCache,
                                  cs: &mut Components,
                                  rng: &mut T,
                                  map_size: (int, int),
                                  player_pos: Position) -> Destination {
    let pos = cs.get::<Position>(e);
    let player_distance = point::tile_distance(pos, player_pos);
    if player_distance < 4 {
        let ai = cs.get::<AI>(e);
        cs.set(AI{state: ai::Aggressive, .. ai}, e);
    }
    match cs.get::<AI>(e).state {
        ai::Aggressive => {
            let r = 8;
            for x in range_inclusive(pos.x - r, pos.x + r) {
                for y in range_inclusive(pos.y - r, pos.y + r) {
                    fail!("TODO: entities_on_pos don't exist");
                    // for monster in cs.entities_on_pos((x, y)) {
                    //     if cs.has::<AI>(monster) {
                    //         let ai = cs.get::<AI>(monster);
                    //         cs.set(AI{state: ai::Aggressive,
                    //                       .. ai}, monster);
                    //     }
                    // }
                }
            }
            Destination{x: player_pos.x, y: player_pos.y}
        }
        ai::Idle => {
            match random_neighbouring_position(rng, pos, cache, cs, map_size) {
                (x, y) => Destination{x: x, y: y}
            }
        }
    }
}


define_system! {
    name: AISystem;
    components(AI, Position);
    resources(player: Entity, position_cache: PositionCache, side: Side,
              world_size: (int, int), rng: ::std::rand::IsaacRng);
    fn process_entity(&mut self, cs: &mut Components, _dt: Duration, e: Entity) {
        let player = *self.player();
        let cache = &*self.position_cache();
        // TODO: having a generic is_alive predicate would be better. How about
        // testing for the presence of Position && (AI || AcceptsUserInput)?
        let player_alive = cs.has::<Position>(player) && cs.has::<Turn>(player);
        if !player_alive { return }
        if *self.side() != Computer { return }

        let world_size = *self.world_size();
        let player_pos = cs.get::<Position>(player);
        let pos = cs.get::<Position>(e);
        let rng = &mut *self.rng();
        let dest = if entity_blocked(pos, cache, cs, world_size) {
            println!("Found a blocked entity: {}", e);
            Destination{x: pos.x, y: pos.y}
        } else {
            match cs.get::<AI>(e).behaviour {
                ai::Individual => {
                    individual_behaviour(e, cache, cs, rng, world_size, player_pos)
                }
                ai::Pack => {
                    hunting_pack_behaviour(e, cache, cs, rng, world_size, player_pos)
                }
            }
        };
        cs.set(dest, e);
    }
}
