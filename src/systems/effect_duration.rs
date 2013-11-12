use components::*;
use super::super::Resources;

pub fn system(entity: ID,
              ecm: &mut ComponentManager,
              res: &mut Resources) {
    if !ecm.has_entity(entity) {return}

    if ecm.has_stunned(entity) {
        let stunned = ecm.get_stunned(entity);
        if stunned.remaining(res.turn) == 0 {
            ecm.remove_stunned(entity);
        }
    }
    if ecm.has_panicking(entity) {
        let panicking = ecm.get_panicking(entity);
        if panicking.remaining(res.turn) == 0 {
            ecm.remove_panicking(entity);
        }
    }
}
