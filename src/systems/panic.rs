use std::rand::Rng;

use components::*;
use super::super::Resources;
use systems::movement::is_walkable;


fn is_wall(pos: Position, ecm: &ComponentManager) -> bool {
    ecm.entities_on_pos(pos).any(|e| {
        ecm.has_background(e) && ecm.has_solid(e)
    })
}

// Can be either an empty place or one with a monster (i.e. blocked but bumpable)
fn random_nonwall_destination<T: Rng>(rng: &mut T,
                                      pos: Position,
                                      ecm: &ComponentManager,
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
    let mut potential_destinations: ~[(int, int)] = ~[];
    for &p in neighbors.iter() {
        let pos = match p { (x, y) => Position{x: x, y: y} };
        if is_walkable(pos, ecm, map_size) || !is_wall(pos, ecm) {
            potential_destinations.push(p)
        }
    }
    if potential_destinations.is_empty() {
        (pos.x, pos.y)  // Nowhere to go
    } else {
        rng.choose(potential_destinations)
    }
}



pub fn system(e: ID,
              ecm: &mut ComponentManager,
              res: &mut Resources) {
    ensure_components!(ecm, e, Panicking, Position, Destination);
    let pos = ecm.get_position(e);
    match random_nonwall_destination(&mut res.rng, pos, ecm, res.world_size) {
        (x, y) => ecm.set_destination(e, Destination{x: x, y: y}),
    }
}
