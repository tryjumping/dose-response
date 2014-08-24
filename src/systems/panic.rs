use std::rand::{IsaacRng, Rng};
use std::time::Duration;

use emhyr::{Components, Entity};
use components::{Background, Destination, Panicking, Position, Solid, UsingItem};
use systems::movement::is_walkable;
use point::Point;


fn is_wall(pos: Position, cs: &Components) -> bool {
    fail!("entities on pos not implemented");
    // ecm.entities_on_pos(pos.coordinates()).any(|e| {
    //     ecm.has::<Background>(e) && ecm.has::<Solid>(e)
    // })
}

// Can be either an empty place or one with a monster (i.e. blocked but bumpable)
fn random_nonwall_destination<T: Rng>(rng: &mut T,
                                      pos: Position,
                                      cs: &Components,
                                      map_size: (int, int)) -> (int, int) {
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
    let mut potential_destinations: Vec<(int, int)> = vec![];
    for &p in neighbors.iter() {
        let pos = match p { (x, y) => Position{x: x, y: y} };
        if is_walkable(pos, cs, map_size) || !is_wall(pos, cs) {
            potential_destinations.push(p)
        }
    }
    match rng.choose(potential_destinations.as_slice()) {
        Some(&p) => p,
        None => (pos.x, pos.y),  // Nowhere to go
    }
}


define_system! {
    name: PanicSystem;
    components(Panicking, Position);
    resources(world_size: (int, int), rng: IsaacRng);
    fn process_entity(&mut self, cs: &mut Components, _dt: Duration, entity: Entity) {
        if cs.has::<UsingItem>(entity) || cs.has::<Destination>(entity) {
            println!("{} panics.", entity);
            // Prevent the item usage
            cs.unset::<UsingItem>(entity);
            // Randomly run around
            let pos = cs.get::<Position>(entity);
            match random_nonwall_destination(&mut *self.rng(), pos, cs, *self.world_size()) {
                (x, y) => cs.set(Destination{x: x, y: y}, entity),
            }
        }
    }
}
