use std::num;
use components::*;
use super::ai;
use super::super::Resources;

pub fn system(e: ID,
              ecm: &mut ComponentManager,
              res: &mut Resources) {
    ensure_components!(ecm, e, Addiction, Attributes, Position, Destination);
    let will = ecm.get_attributes(e).will;
    let pos = ecm.get_position(e);
    let search_radius = 3;  // max irresistibility for a dose is curretnly 3
    let mut doses: ~[ID] = ~[];
    for x in range(pos.x - search_radius, pos.x + search_radius) {
        for y in range(pos.y - search_radius, pos.y + search_radius) {
            for (dose_id, _) in res.map.entities_on_pos((x, y)) {
                let dose = ID(dose_id);
                if !ecm.has_entity(dose) {
                    fail2!("dose system: dose {} on pos {}, {} not in ecm.",
                           dose_id, x, y);
                }
                if !ecm.has_dose(dose) {loop};
                let dose_pos = ecm.get_position(dose);
                let path_to_dose = res.map.find_path((pos.x, pos.y), (dose_pos.x, dose_pos.y));
                let resist_radius = num::max(ecm.get_dose(dose).resist_radius - will, 0);
                let is_irresistible = match path_to_dose {
                    Some(p) => p.len() <= resist_radius,
                    None => false,
                };
                if is_irresistible {
                    doses.push(dose);
                }
            }
        }
    }
    let nearest_dose = do doses.iter().min_by |&dose| {
        ai::distance(&ecm.get_position(*dose), &pos)
    };
    match nearest_dose {
        Some(&dose) => {
            let Position{x, y} = ecm.get_position(dose);
            // We walk the path here to make sure we only move one step at a
            // time.
            match res.map.find_path((pos.x, pos.y), (x, y)) {
                Some(ref mut path) => {
                    let resist_radius = num::max(ecm.get_dose(dose).resist_radius - will, 0);
                    if path.len() <= resist_radius {
                        match path.walk() {
                            Some((x, y)) => {
                                ecm.set_destination(e, Destination{x: x, y: y});
                            }
                            None => unreachable!(),
                        }
                    }
                }
                None => {}
            }
        }
        None => {}
    }
}
