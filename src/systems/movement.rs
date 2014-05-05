use std::cast;
use std::iter::FromIterator;
use libc::{c_int, c_float, c_void};
use emhyr::{ComponentManager, ECM, Entity};

use components::*;
use tcod::Path;
use util::distance;


pub fn is_walkable(pos: Position, ecm: &ECM, map_size: (int, int))
                   -> bool {
    match map_size {
        (width, height) => {
            if pos.x < 0 || pos.y < 0 || pos.x >= width || pos.y >= height {
                return false;
            }
        }
    }
    !is_solid(pos, ecm)
}

fn is_solid(pos: Position, ecm: &ECM) -> bool {
    entities_on_pos(ecm, pos).iter().any(|&e| {
        ecm.has::<Solid>(e)
    })
}

fn entities_on_pos(ecm: &ECM, pos: Position) -> Vec<Entity> {
    // TODO: SLOOOOOOOOOOOOOW
    FromIterator::from_iter(
        ecm.iter().
            filter(|&e|
                   ecm.has::<Position>(e) &&
                   ecm.get::<Position>(e) == pos))
}

struct PathWithUserData {
    to: (int, int),
    ecm: *ECM,
    path: Option<Path>,
}

impl PathWithUserData {
    pub fn len(&self) -> int {
        self.path.get_ref().len()
    }

    pub fn walk(&mut self, recalculate: bool) -> Option<(int, int)> {
        self.path.get_mut_ref().walk(recalculate)
    }
}

// This is unsafe because we're passing the the pointer to ecm to the tcod
// callback and then return an object with the associated callback. Should ecm
// be destroyed before the Path we're returning, things would go wrong. So the
// caller has to make sure that doesn't happen.
pub unsafe fn find_path(from: (int, int), to: (int, int), map_size: (int, int), ecm: *mut ECM)
                 -> Option<~PathWithUserData> {
    let (sx, sy) = from;
    let (dx, dy) = to;
    let (width, height) = map_size;
    if dx < 0 || dy < 0 || dx >= width || dy >= height {
        return None;
    }
    let mut p = ~PathWithUserData {
        to: to,
        ecm: ecm as *ECM,
        path: None,
    };
    let mut path = Path::new_using_function(width, height, Some(cb), p, 1.0);
    match path.find(sx, sy, dx, dy) {
        true => {
            p.path = Some(path);
            Some(p)
        }
        false => None,
    }
}

extern fn cb(xf: c_int, yf: c_int, xt: c_int, yt: c_int, path_data_ptr: *mut c_void) -> c_float {
    // The points should be right next to each other:
    assert!((xf, yf) != (xt, yt) && ((xf-xt) * (yf-yt)).abs() <= 1);
    let p: &PathWithUserData = unsafe { cast::transmute(path_data_ptr) };

    let (dx, dy) = p.to;
    // Succeed if we're at the destination even if it's not walkable:
    if (dx as c_int, dy as c_int) == (xt, yt) {
        1.0
    } else if is_solid(Position{x: xt as int, y: yt as int}, unsafe {cast::transmute(p.ecm)}) {
        0.0
    } else {
        1.0
    }
}

define_system! {
    name: MovementSystem;
    components(Position, Destination, Turn);
    resources(ecm: ECM, world_size: (int, int));
    fn process_entity(&mut self, dt_ms: uint, e: Entity) {
        let mut ecm = self.ecm();
        let turn: Turn = ecm.get(e);
        if turn.ap <= 0 {return}

        let pos: Position = ecm.get(e);
        let dest: Destination = ecm.get(e);
        if (pos.x, pos.y) == (dest.x, dest.y) {
            // Wait (spends an AP but do nothing)
            println!("Entity {:?} waits.", e);
            ecm.set(e, turn.spend_ap(1));
            ecm.remove::<Destination>(e);
        } else if distance(&pos, &Position{x: dest.x, y: dest.y}) == 1 {
            println!("walking 1 step");
            if is_walkable(Position{x: dest.x, y: dest.y}, &*ecm, *self.world_size())  {
                // Move to the cell
                ecm.set(e, turn.spend_ap(1));
                ecm.set(e, Position{x: dest.x, y: dest.y});
                ecm.remove::<Destination>(e);
            } else {  // Bump into the blocked entity
                // TODO: assert there's only one solid entity on pos [x, y]
                for &bumpee in entities_on_pos(&*ecm, Position{x: dest.x, y: dest.y}).iter() {
                    assert!(bumpee != e);
                    match ecm.has::<Solid>(bumpee) {
                        true => {
                            println!("Entity {} bumped into {} at: ({}, {})",
                                     e, bumpee, dest.x, dest.y);
                            ecm.set(e, Bump(bumpee));
                            ecm.remove::<Destination>(e);
                            break;
                        }
                        false => {}
                    }
                }
            }
        } else {  // Farther away than 1 space. Need to use path finding
            println!("walking more than 1 step");
            fail!("TODO: walking more than 1 step");
        //     unsafe {
        //         match find_path((pos.x, pos.y), (dest.x, dest.y), res.world_size, ecm) {
        //             Some(ref mut path) => {
        //                 assert!(path.len() > 1,
        //                         "The path shouldn't be trivial. We already handled that.");
        //                 match path.walk(true) {
        //                     Some((x, y)) => {
        //                         let new_pos = Position{x: x, y: y};
        //                         assert!(distance(&pos, &new_pos) == 1,
        //                                 "The step should be right next to the curret pos.");
        //                         ecm.set(e, turn.spend_ap(1));
        //                         ecm.set(e, new_pos);
        //                     }
        //                     // "The path exists but can't be walked?!"
        //                     None => unreachable!(),
        //                 }
        //             }
        //             None => {
        //                 println!("Entity {:?} cannot find a path so it waits.", e);
        //                 ecm.set(e, turn.spend_ap(1));
        //                 ecm.remove::<Destination>(e);
        //             }
        //         }
        //     }
        }
    }
}
