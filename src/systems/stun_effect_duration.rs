use ecm::{ComponentManager, ECM, Entity};
use components::{Stunned};

define_system! {
    name: StunEffectDurationSystem;
    components(Stunned);
    resources(ecm: ECM, current_turn: int);
    fn process_entity(&mut self, _dt_ms: uint, entity: Entity) {
        let mut ecm = &mut *self.ecm();
        let stunned: Stunned = ecm.get(entity);
        if stunned.remaining(*self.current_turn()) == 0 {
            ecm.remove::<Stunned>(entity);
        }
    }
}
