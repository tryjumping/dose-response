use std::intrinsics::transmute;
use std::time::Duration;
use libc::{c_int, c_float, c_void};
use tcod::Path;

use components::{Bump, Destination, Position, Solid, Turn};
use emhyr::{Components, Entity};
use point;


pub fn is_walkable(pos: Position, cs: &Components, map_size: (int, int))
                   -> bool {
    match map_size {
        (width, height) => {
            if pos.x < 0 || pos.y < 0 || pos.x >= width || pos.y >= height {
                return false;
            }
        }
    }
    !is_solid(pos, cs)
}

fn is_solid(pos: Position, cs: &Components) -> bool {
    fail!("is_solid NOT IMPLEMENTED YET");
    // cs.entities_on_pos((pos.x, pos.y)).any(|e| {
    //     cs.has::<Solid>(e)
    // })
}

struct PathWithUserData<'a> {
    to: (int, int),
    cs: *const Components<'a>,
    path: Option<Path>,
}

impl<'a> PathWithUserData<'a> {
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
pub unsafe fn find_path(from: (int, int), to: (int, int), map_size: (int, int), cs: *const Components)
                 -> Option<Box<PathWithUserData>> {
    let (sx, sy) = from;
    let (dx, dy) = to;
    let (width, height) = map_size;
    if dx < 0 || dy < 0 || dx >= width || dy >= height {
        return None;
    }
    let mut p = box PathWithUserData {
        to: to,
        cs: cs as *const Components,
        path: None,
    };
    fail!("TODO: find_path not working now");
    // let mut path = Path::new_using_function(width, height, Some(cb),
    //                                         transmute::<Box<PathWithUserData>, &PathWithUserData>(p), 1.0);
    // match path.find(sx, sy, dx, dy) {
    //     true => {
    //         p.path = Some(path);
    //         Some(p)
    //     }
    //     false => None,
    // }
}

extern fn cb(xf: c_int, yf: c_int, xt: c_int, yt: c_int, path_data_ptr: *mut c_void) -> c_float {
    // The points should be right next to each other:
    assert!((xf, yf) != (xt, yt) && ((xf-xt) * (yf-yt)).abs() <= 1);
    let p: &PathWithUserData = unsafe { transmute(path_data_ptr) };

    let (dx, dy) = p.to;
    // Succeed if we're at the destination even if it's not walkable:
    if (dx as c_int, dy as c_int) == (xt, yt) {
        1.0
    } else if is_solid(Position{x: xt as int, y: yt as int}, unsafe {transmute(p.cs)}) {
        0.0
    } else {
        1.0
    }
}

define_system! {
    name: MovementSystem;
    components(Position, Destination, Turn);
    resources(world_size: (int, int));
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
            if is_walkable(Position{x: dest.x, y: dest.y}, cs, *self.world_size())  {
                // Move to the cell
                cs.set(turn.spend_ap(1), e);
                cs.set(Position{x: dest.x, y: dest.y}, e);
                cs.unset::<Destination>(e);
            } else {  // Bump into the blocked entity
                // TODO: assert there's only one solid entity on pos [x, y]
                fail!("entities_on_pos not implemented");
                // for bumpee in cs.entities_on_pos((dest.x, dest.y)) {
                //     assert!(bumpee != e);
                //     match cs.has::<Solid>(bumpee) {
                //         true => {
                //             cs.set(Bump(bumpee), e);
                //             cs.unset::<Destination>(e);
                //             break;
                //         }
                //         false => {}
                //     }
                // }
            }
        } else {  // Farther away than 1 space. Need to use path finding
            // TODO: can we minimise the unsafe block to contain just the find_path call?
            unsafe {
                match find_path((pos.x, pos.y), (dest.x, dest.y), *self.world_size(), &*cs) {
                    Some(ref mut path) => {
                        assert!(path.len() > 1,
                                "The path shouldn't be trivial. We already handled that.");
                        match path.walk(true) {
                            Some((x, y)) => {
                                let new_pos = Position{x: x, y: y};
                                assert!(point::tile_distance(pos, new_pos) == 1,
                                        "The step should be right next to the curret pos.");
                                cs.set(turn.spend_ap(1), e);
                                cs.set(new_pos, e);
                            }
                            // "The path exists but can't be walked?!"
                            None => unreachable!(),
                        }
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
}
