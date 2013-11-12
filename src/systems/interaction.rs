use components::*;
use super::combat;
use super::super::Resources;

pub fn system(e: ID,
              ecm: &mut ComponentManager,
              res: &mut Resources) {
    // Only humans can use stuff for now:
    ensure_components!(ecm, e, AcceptsUserInput, Position);
    let pos = match ecm.get_position(e) {Position{x, y} => (x, y)};
    for (entity_map_id, _walkability) in res.map.entities_on_pos(pos) {
        let inter = ID(entity_map_id);
        if e == inter { loop }  // Entity cannot interact with itself
        if !ecm.has_entity(inter) {loop}
        let is_interactive = ecm.has_attribute_modifier(inter) || ecm.has_explosion_effect(inter);
        if !is_interactive {loop}
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
            let (px, py) = pos;
            for x in range(px - radius, px + radius) {
                for y in range(py - radius, py + radius) {
                    for (m_id, _) in res.map.entities_on_pos((x, y)) {
                        let monster = ID(m_id);
                        if ecm.has_entity(monster) && ecm.has_ai(monster) {
                            combat::kill_entity(monster, ecm, &mut res.map);
                        }
                    }
                }
            }
        }
        ecm.remove_position(inter);
        res.map.remove_entity(*inter, pos);
    }
}
