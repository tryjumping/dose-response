use std::time::Duration;
use emhyr::{Components, Entity};
use components::Panicking;

define_system! {
    name: PanicEffectDurationSystem;
    components(Panicking);
    resources(current_turn: int);
    fn process_entity(&mut self, cs: &mut Components, _dt: Duration, entity: Entity) {
        let panicking: Panicking = cs.get(entity);
        if panicking.remaining(*self.current_turn()) == 0 {
            cs.unset::<Panicking>(entity);
        }
    }
}
