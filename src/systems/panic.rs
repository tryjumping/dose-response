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
