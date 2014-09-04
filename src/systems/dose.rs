use std::cmp;
use std::time::Duration;

use components::{Addiction, Attributes, Destination, Dose, Position};
use emhyr::{Components, Entity};
use entity_util::{PositionCache, is_solid};
use point;
use point::Point;

fn cannot_resist(addict: Entity,
                 dose: Entity,
                 cache: &PositionCache,
                 cs: &Components,
                 map_size: (int, int)) -> bool {
    use tcod::AStarPath;
    let pos = cs.get::<Position>(addict);
    let dose_pos = cs.get::<Position>(dose);
    let (width, height) = map_size;
    let mut path = AStarPath::new_from_callback(
        width, height, |from, to| {
            if is_solid(to, cache, cs) {
                0.0
            } else {
                1.0
            }
        }, 1.0);
    path.find(pos.coordinates(), dose_pos.coordinates()) &&
        (path.len() <= resist_radius(addict, dose, cs))
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
                if cannot_resist(addict, dose, cache, cs, world_size) {
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
