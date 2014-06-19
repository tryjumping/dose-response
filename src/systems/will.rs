use components::{AnxietyKillCounter, Attributes};
use ecm::{ComponentManager, ECM, Entity};
use entity_util;


define_system! {
    name: WillSystem;
    components(Attributes);
    resources(ecm: ECM);
    fn process_entity(&mut self, _dt_ms: uint, entity: Entity) {
        let ecm = &mut *self.ecm();
        let attrs = ecm.get::<Attributes>(entity);
        if ecm.has::<AnxietyKillCounter>(entity) {
            let kc = ecm.get::<AnxietyKillCounter>(entity);
            if kc.count >= kc.threshold {
                ecm.set(entity,
                        Attributes{will: attrs.will + 1, .. attrs});
                ecm.set(entity,
                        AnxietyKillCounter{
                            count: kc.threshold - kc.count,
                            .. kc
                        });
            }
        }
        if ecm.get::<Attributes>(entity).will <= 0 {
            entity_util::kill(ecm, entity);
        }
    }
}
