use components::*;
use super::super::Resources;

pub fn system(e: ID,
              ecm: &mut ComponentManager,
              res: &mut Resources) {
    ensure_components!(ecm, e, Turn);
    let turn = ecm.get_turn(e);
    if turn.side == res.side {
        ecm.set(e, Turn{spent_this_tick: 0, .. turn});
    }
}
