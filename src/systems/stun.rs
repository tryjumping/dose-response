use components::*;
use super::super::Resources;

pub fn system(e: ID,
              ecm: &mut ComponentManager,
              _res: &mut Resources) {
    ensure_components!(ecm, e, Stunned, Position, Destination);
    let Position{x, y} = ecm.get_position(e);
    ecm.set_destination(e, Destination{x: x, y: y});
}
