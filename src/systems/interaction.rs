use components::{AcceptsUserInput, Addiction, AttributeModifier, Attributes,
                 Dose, ExplosionEffect, Pickable, Position, InventoryItem};
use ecm::{ComponentManager, ECM, Entity};
use point::Point;
use entity_util;


define_system! {
    name: InteractionSystem;
    components(AcceptsUserInput, Position);
    resources(ecm: ECM);
    fn process_entity(&mut self, _dt_ms: uint, actor: Entity) {
        let ecm = &mut *self.ecm();
        let pos = ecm.get::<Position>(actor);
        for inter in ecm.entities_on_pos(pos.coordinates()) {
            if actor == inter {continue}  // Entity cannot interact with itself
            if !ecm.has_entity(inter) {continue}
            // TODO: only doses and food are interactive for now. If we add more, we
            // should create a new `Interactive` component and test its presense
            // here:
            let is_interactive = ecm.has::<Dose>(inter) || ecm.has::<Pickable>(inter);
            if !is_interactive {continue}
            let is_dose = ecm.has::<Dose>(inter);
            if ecm.has::<AttributeModifier>(inter) {
                let tolerance = if is_dose && ecm.has::<Addiction>(actor) {
                    ecm.get::<Addiction>(actor).tolerance
                } else {
                    0
                };
                if ecm.has::<Attributes>(actor) {
                    let attrs = ecm.get::<Attributes>(actor);
                    let modifier = ecm.get::<AttributeModifier>(inter);
                    ecm.set(actor, Attributes{
                        state_of_mind: attrs.state_of_mind + modifier.state_of_mind - tolerance,
                        will: attrs.will + modifier.will,
                    });
                }
            }
            if is_dose {
                if ecm.has::<Addiction>(actor) {
                    let addiction = ecm.get::<Addiction>(actor);
                    let dose = ecm.get::<Dose>(inter);
                    ecm.set(actor, Addiction{
                        tolerance: addiction.tolerance + dose.tolerance_modifier,
                        .. addiction});
                }
                if ecm.has::<ExplosionEffect>(inter) {
                    let radius = ecm.get::<ExplosionEffect>(inter).radius;
                    entity_util::explosion(ecm, pos, radius);
                }
            } else {
                ecm.set(inter, InventoryItem{owner: actor});
                println!("Item {} picked up by {}", inter, actor);
            }
            ecm.remove::<Position>(inter);
        }
    }
}
