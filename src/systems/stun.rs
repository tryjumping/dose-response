use components::*;
use super::super::Resources;

pub fn system(e: ID,
              ecm: &mut ComponentManager,
              _res: &mut Resources) {
    ensure_components!(ecm, e, Stunned, Position);
    if ecm.has_destination(e) {
        let Position{x, y} = ecm.get::<Position>(e);
        ecm.set_destination(e, Destination{x: x, y: y});
    } else if ecm.has_using_item(e) {
        println!("Entity {:?} cannot use items because it's stunned.", e);
        ecm.remove_using_item(e);
    }
}
