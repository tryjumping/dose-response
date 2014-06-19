use ecm::{ComponentManager, ECM, Entity};
use components::{Addiction, Attributes};
use entity_util;


define_system! {
    name: AddictionSystem;
    components(Addiction, Attributes);
    resources(ecm: ECM, current_turn: int);
    fn process_entity(&mut self, _dt_ms: uint, entity: Entity) {
        let ecm = &mut *self.ecm();
        let addiction: Addiction = ecm.get(entity);
        let attr: Attributes = ecm.get(entity);
        let current_turn = *self.current_turn();
        if current_turn > addiction.last_turn {
            ecm.set(entity, Attributes{
                state_of_mind: attr.state_of_mind - addiction.drop_per_turn,
                .. attr
            });
            ecm.set(entity, Addiction{last_turn: current_turn, .. addiction});
        };
        let som = ecm.get::<Attributes>(entity).state_of_mind;
        if som <= 0 || som >= 100 {
            entity_util::kill(ecm, entity);
        }
    }
}
