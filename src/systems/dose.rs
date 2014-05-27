use std::num;
use std::iter::range_inclusive;

use components::{Destination, Position};
use super::ai;
use super::super::Resources;
use systems::movement::find_path;
use util::Deref;

fn is_irresistible(addict: ID,
                   dose: ID,
                   ecm: &ComponentManager,
                   map_size: (int, int)) -> bool {
    let pos = ecm.get::<Position>(addict);
    let dose_pos = ecm.get::<Position>(dose);
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
    let pos = ecm.get::<Position>(e);
    let search_radius = 3;  // max irresistibility for a dose is curretnly 3
    let mut doses: ~[ID] = ~[];
    for (x, y) in points_within_radius(entity_position) {
        for dose in ecm.entities_on_pos((x, y)) {
            if !ecm.has_entity(dose) {
                fail!("dose system: dose {:?} on pos {:?} not in ecm.", dose, (x, y));
            }
            if !ecm.has_dose(dose) {continue};
            if is_irresistible(e, dose, ecm, res.world_size) {
                doses.push(dose);
            }
        }
    }
    for x in range_inclusive(pos.x - search_radius, pos.x + search_radius) {
        for y in range_inclusive(pos.y - search_radius, pos.y + search_radius) {
        }
    }
    let nearest_dose = doses.iter().min_by(|&dose| {
        ai::distance(&ecm.get::<Position>(*dose), &pos)
    });
    match nearest_dose {
        Some(&dose) => {
            let Position{x, y} = ecm.get::<Position>(dose);
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
