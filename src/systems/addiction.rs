use std::time::Duration;
use emhyr::{Components, Entity};
use components::{Addiction, Attributes};
use entity_util;


define_system! {
    name: AddictionSystem;
    components(Addiction, Attributes);
    resources(current_turn: int);
    fn process_entity(&mut self, cs: &mut Components, _dt: Duration, entity: Entity) {
        let addiction: Addiction = cs.get(entity);
        let attr: Attributes = cs.get(entity);
        let current_turn = *self.current_turn();
        if current_turn > addiction.last_turn {
            cs.set(Attributes{
                state_of_mind: attr.state_of_mind - addiction.drop_per_turn,
                .. attr
            }, entity);
            cs.set(Addiction{last_turn: current_turn, .. addiction}, entity);
        };
        let som = cs.get::<Attributes>(entity).state_of_mind;
        if som <= 0 || som >= 100 {
            entity_util::kill(cs, entity);
        }
    }
}
