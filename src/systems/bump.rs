use ecm::{ComponentManager, ECM, Entity};
use components::{AttackTarget, Bump, Turn};


define_system! {
    name: BumpSystem;
    components(Bump, Turn);
    resources(ecm: ECM);
    fn process_entity(&mut self, dt_ms: uint, entity: Entity) {
        let mut ecm = &mut *self.ecm();
        let Bump(bumpee) = ecm.get::<Bump>(entity);
        ecm.remove::<Bump>(entity);
        if !ecm.has_entity(bumpee) {return}

        let opposing_sides = (ecm.has::<Turn>(bumpee)
                              && ecm.get::<Turn>(bumpee).side != ecm.get::<Turn>(entity).side);
        if opposing_sides {
            println!("Entity {} attacks {}.", entity, bumpee);
            ecm.set(entity, AttackTarget(bumpee));
        } else {
            println!("Entity {} hits the wall.", entity);
        }
    }
}
