use ecm::{ComponentManager, ECM, Entity};
use components::Panicking;

define_system! {
    name: PanicEffectDurationSystem;
    components(Panicking);
    resources(ecm: ECM, current_turn: int);
    fn process_entity(&mut self, _dt_ms: uint, entity: Entity) {
        let ecm = &mut *self.ecm();
        let panicking: Panicking = ecm.get(entity);
        if panicking.remaining(*self.current_turn()) == 0 {
            ecm.remove::<Panicking>(entity);
        }
    }
}
