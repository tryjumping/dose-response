use std::cmp;
use std::iter::range_inclusive;

use components::{Addiction, Attributes, Destination, Dose, Position};
use ecm::{ComponentManager, ECM, Entity};
use systems::movement::find_path;
use point;

fn is_irresistible(addict: Entity,
                   dose: Entity,
                   ecm: &mut ECM,
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

fn resist_radius(addict: Entity, dose: Entity, ecm: &ECM) -> int {
    let will = ecm.get::<Attributes>(addict).will;
    cmp::max(ecm.get::<Dose>(dose).resist_radius - will, 0)
}

define_system! {
    name: DoseSystem;
    components(Addiction, Attributes, Position, Destination);
    resources(ecm: ECM, world_size: (int, int));
    fn process_entity(&mut self, dt_ms: uint, addict: Entity) {
        let ecm = &mut *self.ecm();
        let world_size = *self.world_size();
        let pos = ecm.get::<Position>(addict);
        let search_radius = 3;  // max irresistibility for a dose is curretnly 3
        let mut doses: Vec<Entity> = Vec::new();
        for (x, y) in point::points_within_radius(pos, search_radius) {
            for dose in ecm.entities_on_pos((x, y)) {
                if !ecm.has_entity(dose) {
                    fail!("dose system: dose {:?} on pos {:?} not in ecm.", dose, (x, y));
                }
                if !ecm.has::<Dose>(dose) {continue};
                if is_irresistible(addict, dose, ecm, world_size) {
                    doses.push(dose);
                }
            }
        }
        let nearest_dose = doses.iter().min_by(|&dose| {
            point::tile_distance(ecm.get::<Position>(*dose), pos)
        });
        match nearest_dose {
            Some(&dose) => {
                let Position{x, y} = ecm.get::<Position>(dose);
                unsafe {
                    // We walk the path here to make sure we only move one step at a
                    // time.
                    match find_path((pos.x, pos.y), (x, y), world_size, ecm) {
                        Some(ref mut path) => {
                            if path.len() <= resist_radius(addict, dose, ecm) {
                                match path.walk(true) {
                                    Some((x, y)) => {
                                        ecm.set(addict, Destination{x: x, y: y});
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
}
