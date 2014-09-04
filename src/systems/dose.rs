use std::cmp;
use std::time::Duration;

use components::{Addiction, Attributes, Destination, Dose, Position};
use emhyr::{Components, Entity};
use entity_util::PositionCache;
use point;

fn is_irresistible(addict: Entity,
                   dose: Entity,
                   cs: &Components,
                   map_size: (int, int)) -> bool {
    let pos = cs.get::<Position>(addict);
    let dose_pos = cs.get::<Position>(dose);
    unsafe {
        fail!("TODO; PATH FINDING IN DOSE");
        // match find_path((pos.x, pos.y), (dose_pos.x, dose_pos.y), map_size, cs) {
        //     Some(p) => p.len() <= resist_radius(addict, dose, cs),
        //     None => false,
        // }
    }
}

fn resist_radius(addict: Entity, dose: Entity, cs: &Components) -> int {
    let will = cs.get::<Attributes>(addict).will;
    cmp::max(cs.get::<Dose>(dose).resist_radius - will, 0)
}

define_system! {
    name: DoseSystem;
    components(Addiction, Attributes, Position, Destination);
    resources(position_cache: PositionCache, world_size: (int, int));
    fn process_entity(&mut self, cs: &mut Components, _dt: Duration, addict: Entity) {
        let world_size = *self.world_size();
        let cache = &*self.position_cache();
        let pos = cs.get::<Position>(addict);
        let search_radius = 3;  // max irresistibility for a dose is curretnly 3
        let mut doses: Vec<Entity> = vec![];
        for (x, y) in point::points_within_radius(pos, search_radius) {
            for dose in cache.entities_on_pos((x, y)) {
                if !cs.has::<Dose>(dose) {continue};
                if is_irresistible(addict, dose, cs, world_size) {
                    doses.push(dose);
                }
            }
        }
        let nearest_dose = doses.iter().min_by(|&dose| {
            point::tile_distance(cs.get::<Position>(*dose), pos)
        });
        match nearest_dose {
            Some(&dose) => {
                let Position{x, y} = cs.get::<Position>(dose);
                unsafe {
                    // We walk the path here to make sure we only move one step at a
                    // time.
                    fail!("TODO: path finding in dose");
                    // match find_path((pos.x, pos.y), (x, y), world_size, &*cs) {
                    //     Some(ref mut path) => {
                    //         if path.len() <= resist_radius(addict, dose, cs) {
                    //             match path.walk(true) {
                    //                 Some((x, y)) => {
                    //                     cs.set(Destination{x: x, y: y}, addict);
                    //                 }
                    //                 None => unreachable!(),
                    //             }
                    //         }
                    //     }
                    //     None => {}
                    // }
                }
            }
            None => {}
        }
    }
}
