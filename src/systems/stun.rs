use std::time::Duration;
use emhyr::{Components, Entity};
use components::{Destination, Position, Stunned, UsingItem};


define_system! {
    name: StunSystem;
    components(Stunned, Position);
    resources(player: Entity);
    fn process_entity(&mut self, cs: &mut Components, _dt: Duration, entity: Entity) {
        if cs.has::<Destination>(entity) {
            let Position{x, y} = cs.get::<Position>(entity);
            cs.set(Destination{x: x, y: y}, entity);
        } else if cs.has::<UsingItem>(entity) {
            println!("{} cannot use items because it's stunned.", entity);
            cs.unset::<UsingItem>(entity);
        }
    }
}
