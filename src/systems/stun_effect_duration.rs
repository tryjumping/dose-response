use std::time::Duration;
use emhyr::{Components, Entity};
use components::{Stunned};

define_system! {
    name: StunEffectDurationSystem;
    components(Stunned);
    resources(current_turn: int);
    fn process_entity(&mut self, cs: &mut Components, _dt: Duration, entity: Entity) {
        let stunned: Stunned = cs.get(entity);
        if stunned.remaining(*self.current_turn()) == 0 {
            cs.unset::<Stunned>(entity);
        }
    }
}
