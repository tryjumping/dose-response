use std::time::Duration;
use components::{Bump, Destination, Position, Solid, Turn};
use emhyr::{Components, Entity};
use point;
use point::Point;
use entity_util::{is_walkable};
use tcod::AStarPath;


pub fn walk_one_step<P1: Point, P2: Point>(source: P1, destination: P2, world_size: (int, int),
                     cache: &PositionCache, cs: &Components) -> Option<(int, int)> {
    let (width, height) = world_size;
    let dest_coords = destination.coordinates();
    let mut path = AStarPath::new_from_callback(
        width, height,
        |&mut: _from: (int, int), to: (int, int)| -> f32 {
            use entity_util;
            // The destination is probably a monster or a player (who are solid).
            // Count that area as walkable.
            if to == dest_coords {
                1.0
            } else if entity_util::is_solid(to, cache, cs) {
                0.0
            } else {
                1.0
            }
        }, 1.0);
    path.find(source.coordinates(), destination.coordinates());
    assert!(path.len() != 1, "The path shouldn't be trivial. We already handled that.");
    path.walk_one_step(true)
}

define_system! {
    name: MovementSystem;
    components(Position, Destination, Turn);
    resources(position_cache: PositionCache, world_size: (int, int));
    fn process_entity(&mut self, cs: &mut Components, _dt: Duration, e: Entity) {
        let turn: Turn = cs.get(e);
        if turn.ap <= 0 {return}

        let pos: Position = cs.get(e);
        let dest: Destination = cs.get(e);
        if (pos.x, pos.y) == (dest.x, dest.y) {
            // Wait (spends an AP but do nothing)
            println!("{} waits.", e);
            cs.set(turn.spend_ap(1), e);
            cs.unset::<Destination>(e);
        } else if point::tile_distance(pos, dest) == 1 {
            let walkable = is_walkable(dest, &*self.position_cache(), cs, *self.world_size());
            if walkable {
                // Move to the cell
                cs.set(turn.spend_ap(1), e);
                cs.set(Position{x: dest.x, y: dest.y}, e);
                cs.unset::<Destination>(e);
            } else {  // Bump into the blocked entity
                // TODO: assert there's only one solid entity on pos [x, y]
                for bumpee in self.position_cache().entities_on_pos(dest) {
                    assert!(bumpee != e);
                    match cs.has::<Solid>(bumpee) {
                        true => {
                            cs.set(Bump(bumpee), e);
                            cs.unset::<Destination>(e);
                            break;
                        }
                        false => {}
                    }
                }
            }
        } else {  // Farther away than 1 space. Need to use path finding
            println!("{} finding path from: {} to {}", e, pos, dest);
            let step = walk_one_step(pos, dest, *self.world_size(), &*self.position_cache(), cs);
            match step {
                Some((x, y)) => {
                    let new_pos = Position{x: x, y: y};
                    assert!(point::tile_distance(pos, new_pos) == 1,
                            "The step should be right next to the curret pos.");
                    cs.set(turn.spend_ap(1), e);
                    cs.set(new_pos, e);
                }
                None => {
                    println!("{} cannot find a path so it waits.", e);
                    cs.set(turn.spend_ap(1), e);
                    cs.unset::<Destination>(e);
                }
            }
        }
    }
}
