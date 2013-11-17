use std::cast;
use std::libc::{c_int, c_float, c_void};

use components::*;
use super::ai;
use super::super::Resources;
use tcod::Path;


pub fn is_walkable(pos: Position, ecm: &ComponentManager, map_size: (int, int))
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

fn is_solid(pos: Position, ecm: &ComponentManager) -> bool {
    do ecm.entities_on_pos(pos).any |e| {
        ecm.has_solid(e)
    }
}

struct PathWithUserData {
    priv to: (int, int),
    priv ecm: *ComponentManager,
    priv path: Option<Path>,
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
pub unsafe fn find_path(from: (int, int), to: (int, int), map_size: (int, int), ecm: &ComponentManager)
                 -> Option<~PathWithUserData> {
    let (sx, sy) = from;
    let (dx, dy) = to;
    let (width, height) = map_size;
    if dx < 0 || dy < 0 || dx >= width || dy >= height {
        return None;
    }
    let mut p = ~PathWithUserData {
        to: to,
        ecm: ecm as *ComponentManager,
        path: None,
    };
    let mut path = Path::new_using_function(width, height, cb, p, 1.0);
    match path.find(sx, sy, dx, dy) {
        true => {
            p.path = Some(path);
            Some(p)
        }
        false => None,
    }
}

extern fn cb(xf: c_int, yf: c_int, xt: c_int, yt: c_int, path_data_ptr: *c_void) -> c_float {
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


pub fn system(e: ID,
              ecm: &mut ComponentManager,
              res: &mut Resources) {
    ensure_components!(ecm, e, Position, Destination, Turn);
    let turn = ecm.get_turn(e);
    if turn.ap <= 0 {return}

    let pos = ecm.get_position(e);
    let dest = ecm.get_destination(e);
    let map_size = (res.map.width, res.map.height);
    if (pos.x, pos.y) == (dest.x, dest.y) {
        // Wait (spends an AP but do nothing)
        println!("Entity {} waits.", *e);
        ecm.set_turn(e, turn.spend_ap(1));
        ecm.remove_destination(e);
    } else if ai::distance(&pos, &Position{x: dest.x, y: dest.y}) == 1 {
        if is_walkable(Position{x: dest.x, y: dest.y}, ecm, map_size)  {  // Move to the cell
            ecm.set_turn(e, turn.spend_ap(1));
            ecm.set_position(e, Position{x: dest.x, y: dest.y});
            ecm.remove_destination(e);
        } else {  // Bump into the blocked entity
            // TODO: assert there's only one solid entity on pos [x, y]
            for bumpee in ecm.entities_on_pos(Position{x: dest.x, y: dest.y}) {
                assert!(bumpee != e);
                match ecm.has_solid(bumpee) {
                    true => {
                        println!("Entity {} bumped into {} at: ({}, {})",
                                 *e, *bumpee, dest.x, dest.y);
                        ecm.set_bump(e, Bump(bumpee));
                        ecm.remove_destination(e);
                        break;
                    }
                    false => {}
                }
            }
        }
    } else {  // Farther away than 1 space. Need to use path finding
        unsafe {
            match find_path((pos.x, pos.y), (dest.x, dest.y), map_size, ecm) {
                Some(ref mut path) => {
                    assert!(path.len() > 1,
                            "The path shouldn't be trivial. We already handled that.");
                    match path.walk(true) {
                        Some((x, y)) => {
                            let new_pos = Position{x: x, y: y};
                            assert!(ai::distance(&pos, &new_pos) == 1,
                                    "The step should be right next to the curret pos.");
                            ecm.set_turn(e, turn.spend_ap(1));
                            ecm.set_position(e, new_pos);
                        }
                        // "The path exists but can't be walked?!"
                        None => unreachable!(),
                    }
                }
                None => {
                    println!("Entity {} cannot find a path so it waits.", *e);
                    ecm.set_turn(e, turn.spend_ap(1));
                    ecm.remove_destination(e);
                }
            }
        }
    }
}
