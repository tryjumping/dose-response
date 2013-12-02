use components::*;
use super::combat;
use super::super::Resources;

pub fn system(e: ID,
              ecm: &mut ComponentManager,
              _res: &mut Resources) {
    // Only humans can use stuff for now:
    ensure_components!(ecm, e, AcceptsUserInput, Position);
    let pos = ecm.get_position(e);
    for inter in ecm.entities_on_pos(pos) {
        if e == inter {continue}  // Entity cannot interact with itself
        if !ecm.has_entity(inter) {continue}
        // TODO: only doses are interactive for now. If we add more, we should
        // create a new `Interactive` component and test its presense here:
        let is_interactive = ecm.has_dose(inter);
        if !is_interactive {continue}
        let is_dose = ecm.has_dose(inter);
        if ecm.has_attribute_modifier(inter) {
            let tolerance = if is_dose && ecm.has_addiction(e) {
                ecm.get_addiction(e).tolerance
            } else {
                0
            };
            if ecm.has_attributes(e) {
                let attrs = ecm.get_attributes(e);
                let modifier = ecm.get_attribute_modifier(inter);
                ecm.set_attributes(e, Attributes{
                        state_of_mind: attrs.state_of_mind + modifier.state_of_mind - tolerance,
                        will: attrs.will + modifier.will,
                    });
            }
        }
        if is_dose {
            if ecm.has_addiction(e) {
                let addiction = ecm.get_addiction(e);
                let dose = ecm.get_dose(inter);
                ecm.set_addiction(e, Addiction{
                        tolerance: addiction.tolerance + dose.tolerance_modifier,
                        .. addiction});
            }
        }
        if ecm.has_explosion_effect(inter) {
            let radius = ecm.get_explosion_effect(inter).radius;
            for x in range(pos.x - radius, pos.x + radius) {
                for y in range(pos.y - radius, pos.y + radius) {
                    for monster in ecm.entities_on_pos(Position{x: x, y: y}) {
                        if ecm.has_entity(monster) && ecm.has_ai(monster) {
                            combat::kill_entity(monster, ecm);
                        }
                    }
                }
            }
        }
        ecm.remove_position(inter);
    }
}
