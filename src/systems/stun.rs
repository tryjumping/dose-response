use ecm::{ComponentManager, ECM, Entity};
use components::{Destination, Position, Stunned, UsingItem};


define_system! {
    name: StunSystem;
    components(Stunned, Position);
    resources(ecm: ECM);
    fn process_entity(&mut self, dt_ms: uint, entity: Entity) {
        let mut ecm = &mut *self.ecm();
        if ecm.has::<Destination>(entity) {
            let Position{x, y} = ecm.get::<Position>(entity);
            ecm.set(entity, Destination{x: x, y: y});
        } else if ecm.has::<UsingItem>(entity) {
            println!("{} cannot use items because it's stunned.", entity);
            ecm.remove::<UsingItem>(entity);
        }
    }
}
