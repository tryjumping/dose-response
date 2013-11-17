use std::num;

use components::*;
use super::super::Resources;

pub fn precise_distance(p1: (int, int), p2: (int, int)) -> int {
    let (x1, y1) = p1;
    let (x2, y2) = p2;
    let a = num::pow(num::abs(x1 - x2) as float, 2f);
    let b = num::pow(num::abs(y1 - y2) as float, 2f);
    num::sqrt(a + b).floor() as int
}

pub fn system(e: ID,
              ecm: &mut ComponentManager,
              res: &mut Resources) {
    if e != res.player_id {return}
    ensure_components!(ecm, e, AcceptsUserInput, Position, Exploration, Attributes);
    let pos = ecm.get_position(e);
    let exploration = ecm.get_exploration(e);
    let attrs = ecm.get_attributes(e);
    let radius = (4 * attrs.state_of_mind + 293) / 99;  // range(3, 8)
    if radius != exploration.radius {
        ecm.set_exploration(e, Exploration{radius: radius});
    }
    for x in range(pos.x - radius, pos.x + radius) {
        for y in range(pos.y - radius, pos.y + radius) {
            if precise_distance((pos.x, pos.y), (x, y)) <= radius {
                for exploree in ecm.entities_on_pos(Position{x: x, y: y}) {
                    if ecm.has_explored(exploree) {
                        ecm.set_explored(exploree, Explored);
                    }
                }
            }
        }
    }
}
