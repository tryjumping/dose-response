use std::time::Duration;
use emhyr::{Components, Entity};
use components::{AttackTarget, Bump, Turn};


define_system! {
    name: BumpSystem;
    components(Bump, Turn);
    resources(player: Entity);
    fn process_entity(&mut self, cs: &mut Components, _dt: Duration, entity: Entity) {
        let Bump(bumpee) = cs.get::<Bump>(entity);
        cs.unset::<Bump>(entity);

        let opposing_sides = cs.has::<Turn>(bumpee) &&
            cs.get::<Turn>(bumpee).side != cs.get::<Turn>(entity).side;
        if opposing_sides {
            println!("{} attacks {}.", entity, bumpee);
            cs.set(AttackTarget(bumpee), entity);
        } else {
            println!("{} hits the wall.", entity);
        }
    }
}
