use std::time::Duration;
use components::{AcceptsUserInput, Addiction, AttributeModifier, Attributes,
                 Dose, ExplosionEffect, Pickable, Position, InventoryItem};
use emhyr::{Components, Entity};
use point::Point;
use entity_util;


define_system! {
    name: InteractionSystem;
    components(AcceptsUserInput, Position);
    resources(player: Entity);
    fn process_entity(&mut self, cs: &mut Components, _dt: Duration, actor: Entity) {
        let pos = cs.get::<Position>(actor);
        fail!("entities_on_pos")
        // for inter in cs.entities_on_pos(pos.coordinates()) {
        //     if actor == inter {continue}  // Entity cannot interact with itself
        //     // TODO: only doses and food are interactive for now. If we add more, we
        //     // should create a new `Interactive` component and test its presense
        //     // here:
        //     let is_interactive = cs.has::<Dose>(inter) || cs.has::<Pickable>(inter);
        //     if !is_interactive {continue}
        //     let is_dose = cs.has::<Dose>(inter);
        //     if cs.has::<AttributeModifier>(inter) {
        //         let tolerance = if is_dose && cs.has::<Addiction>(actor) {
        //             cs.get::<Addiction>(actor).tolerance
        //         } else {
        //             0
        //         };
        //         if cs.has::<Attributes>(actor) {
        //             let attrs = cs.get::<Attributes>(actor);
        //             let modifier = cs.get::<AttributeModifier>(inter);
        //             cs.set(Attributes{
        //                 state_of_mind: attrs.state_of_mind + modifier.state_of_mind - tolerance,
        //                 will: attrs.will + modifier.will,
        //             }, actor);
        //         }
        //     }
        //     if is_dose {
        //         if cs.has::<Addiction>(actor) {
        //             let addiction = cs.get::<Addiction>(actor);
        //             let dose = cs.get::<Dose>(inter);
        //             cs.set(Addiction{
        //                 tolerance: addiction.tolerance + dose.tolerance_modifier,
        //                 .. addiction}, actor);
        //         }
        //         if cs.has::<ExplosionEffect>(inter) {
        //             let radius = cs.get::<ExplosionEffect>(inter).radius;
        //             entity_util::explosion(cs, pos, radius);
        //         }
        //     } else {
        //         cs.set(InventoryItem{owner: actor}, inter);
        //         println!("Item {} picked up by {}", inter, actor);
        //     }
        //     cs.unset::<Position>(inter);
        // }
    }
}
