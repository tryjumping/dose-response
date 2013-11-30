use std::num;
use components::*;
use super::ai;
use super::super::Resources;
use systems::movement::find_path;

fn is_irresistible(addict: ID,
                   dose: ID,
                   ecm: &ComponentManager,
                   map_size: (int, int)) -> bool {
    let pos = ecm.get_position(addict);
    let dose_pos = ecm.get_position(dose);
    unsafe {
        match find_path((pos.x, pos.y), (dose_pos.x, dose_pos.y), map_size, ecm) {
            Some(p) => p.len() <= resist_radius(addict, dose, ecm),
            None => false,
        }
    }
}

fn resist_radius(addict: ID, dose: ID, ecm: &ComponentManager) -> int {
    let will = ecm.get_attributes(addict).will;
    num::max(ecm.get_dose(dose).resist_radius - will, 0)
}

pub fn system(e: ID,
              ecm: &mut ComponentManager,
              res: &mut Resources) {
    ensure_components!(ecm, e, Addiction, Attributes, Position, Destination);
    let pos = ecm.get_position(e);
    let search_radius = 3;  // max irresistibility for a dose is curretnly 3
    let mut doses: ~[ID] = ~[];
    for x in range(pos.x - search_radius, pos.x + search_radius) {
        for y in range(pos.y - search_radius, pos.y + search_radius) {
            for dose in ecm.entities_on_pos(Position{x: x, y: y}) {
                if !ecm.has_entity(dose) {
                    fail2!("dose system: dose {} on pos {}, {} not in ecm.",
                           *dose, x, y);
                }
                if !ecm.has_dose(dose) {continue};
                if is_irresistible(e, dose, ecm, res.world_size) {
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
            unsafe {
                // We walk the path here to make sure we only move one step at a
                // time.
                match find_path((pos.x, pos.y), (x, y), res.world_size, ecm) {
                    Some(ref mut path) => {
                        if path.len() <= resist_radius(e, dose, ecm) {
                            match path.walk(true) {
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
        }
        None => {}
    }
}
