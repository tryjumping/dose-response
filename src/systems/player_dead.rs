use components::{ComponentManager, ID};
use super::super::Resources;
use util::Deref;

pub fn system(id: ID,
              ecm: &mut ComponentManager,
              res: &mut Resources) {
    let player = res.player_id;
    if !ecm.has_entity(player) {
        fail!("Could not find the Player entity (id: {})", res.player_id.deref())
    }
    let player_dead = !ecm.has_position(player) || !ecm.has_turn(player);
    if player_dead {
        match ecm.has_ai(id) {
            true => ecm.remove_ai(id),
            false => (),
        }
    }
}
