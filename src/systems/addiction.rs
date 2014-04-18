use components::*;
use super::combat;
use super::super::Resources;

pub fn system(e: ID,
              ecm: &mut ComponentManager,
              res: &mut Resources) {
    ensure_components!(ecm, e, Addiction, Attributes);
    let addiction = ecm.get_addiction(e);
    let attr = ecm.get_attributes(e);
    if res.turn > addiction.last_turn {
        ecm.set(e, Attributes{
                state_of_mind: attr.state_of_mind - addiction.drop_per_turn,
                .. attr
            });
        ecm.set(e, Addiction{last_turn: res.turn, .. addiction});
    };
    let som = ecm.get_attributes(e).state_of_mind;
    if som <= 0 || som >= 100 {
        combat::kill_entity(e, ecm);
    }
}
