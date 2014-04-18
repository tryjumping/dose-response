use std::num;
use std::iter::range_inclusive;

use components::*;
use super::super::Resources;
use super::addiction_graphics::intoxication_state::*;

pub fn system(e: ID,
              ecm: &mut ComponentManager,
              res: &mut Resources) {
    if e != res.player_id {return}
    ensure_components!(ecm, e, AcceptsUserInput, Position, Exploration, Attributes);
    let pos = ecm.get::<Position>(e);
    let exploration = ecm.get_exploration(e);
    let attrs = ecm.get_attributes(e);
    let radius = match IntoxicationState::from_int(attrs.state_of_mind) {
        Exhausted | DeliriumTremens => 4,
        Withdrawal => 5,
        Sober => 6,
        High => 7,
        VeryHigh | Overdosed => 8,
    };
    if radius != exploration.radius {
        ecm.set(e, Exploration{radius: radius});
    }
    for x in range_inclusive(pos.x - radius, pos.x + radius) {
        for y in range_inclusive(pos.y - radius, pos.y + radius) {
            if precise_distance((pos.x, pos.y), (x, y)) <= radius {
                for exploree in ecm.entities_on_pos(Position{x: x, y: y}) {
                    if ecm.has_tile(exploree) && ecm.has_position(exploree) {
                        ecm.set(exploree, Explored);
                    }
                }
            }
        }
    }
}
