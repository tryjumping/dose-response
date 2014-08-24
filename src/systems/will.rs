use std::time::Duration;
use components::{AnxietyKillCounter, Attributes};
use emhyr::{Components, Entity};
use entity_util;


define_system! {
    name: WillSystem;
    components(Attributes);
    resources(player: Entity);
    fn process_entity(&mut self, cs: &mut Components, _dt: Duration, entity: Entity) {
        let attrs = cs.get::<Attributes>(entity);
        if cs.has::<AnxietyKillCounter>(entity) {
            let kc = cs.get::<AnxietyKillCounter>(entity);
            if kc.count >= kc.threshold {
                cs.set(Attributes{will: attrs.will + 1, .. attrs}, entity);
                cs.set(AnxietyKillCounter{
                            count: kc.threshold - kc.count,
                            .. kc
                        }, entity);
            }
        }
        if cs.get::<Attributes>(entity).will <= 0 {
            entity_util::kill(cs, entity);
        }
    }
}
