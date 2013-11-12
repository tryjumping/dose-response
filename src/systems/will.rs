use components::*;
use super::combat;
use super::super::Resources;

pub fn system(e: ID,
              ecm: &mut ComponentManager,
              res: &mut Resources) {
    ensure_components!(ecm, e, Attributes);
    let attrs = ecm.get_attributes(e);

    if ecm.has_anxiety_kill_counter(e) {
        let kc = ecm.get_anxiety_kill_counter(e);
        if kc.count >= kc.threshold {
            ecm.set_attributes(e,
                               Attributes{will: attrs.will + 1, .. attrs});
            ecm.set_anxiety_kill_counter(e,
                                         AnxietyKillCounter{
                    count: kc.threshold - kc.count,
                    .. kc
                });
        }
    }
    if ecm.get_attributes(e).will <= 0 {
        combat::kill_entity(e, ecm, &mut res.map);
    }
}
