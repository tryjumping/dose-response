use components::*;
use super::super::Resources;

pub fn system(e: ID,
              ecm: &mut ComponentManager,
              _res: &mut Resources) {
    ensure_components!(ecm, e, Bump)
        let bumpee = *ecm.get_bump(e);
    ecm.remove_bump(e);
    if !ecm.has_entity(bumpee) {return}
    let different_sides = (ecm.has_turn(bumpee) && ecm.has_turn(e)
                           && ecm.get_turn(bumpee).side != ecm.get_turn(e).side);
    if different_sides {
        println!("Entity {} attacks {}.", *e, *bumpee);
        ecm.set_attack_target(e, AttackTarget(bumpee));
    } else {
        println!("Entity {} hits the wall.", *e);
    }
}
