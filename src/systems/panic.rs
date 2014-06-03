use rand::{IsaacRng, Rng};

use ecm::{ComponentManager, ECM, Entity};
use components::{Background, Destination, Panicking, Position, Solid, UsingItem};
use systems::movement::is_walkable;
use point::Point;


fn is_wall(pos: Position, ecm: &ECM) -> bool {
    ecm.entities_on_pos(pos.coordinates()).any(|e| {
        ecm.has::<Background>(e) && ecm.has::<Solid>(e)
    })
}

// Can be either an empty place or one with a monster (i.e. blocked but bumpable)
fn random_nonwall_destination<T: Rng>(rng: &mut T,
                                      pos: Position,
                                      ecm: &ECM,
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
        if is_walkable(pos, ecm, map_size) || !is_wall(pos, ecm) {
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
    resources(ecm: ECM, world_size: (int, int), rng: IsaacRng);
    fn process_entity(&mut self, dt_ms: uint, entity: Entity) {
        let mut ecm = &mut *self.ecm();
        if ecm.has::<UsingItem>(entity) || ecm.has::<Destination>(entity) {
            println!("{} panics.", entity);
            // Prevent the item usage
            ecm.remove::<UsingItem>(entity);
            // Randomly run around
            let pos = ecm.get::<Position>(entity);
            match random_nonwall_destination(&mut *self.rng(), pos, ecm, *self.world_size()) {
                (x, y) => ecm.set(entity, Destination{x: x, y: y}),
            }
        }
    }
}
