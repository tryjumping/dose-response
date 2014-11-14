use std::time::Duration;
use components::{AcceptsUserInput, Position, Exploration, Attributes, Explored, Tile};
use emhyr::{Components, Entity, Entities};
use point;
use super::addiction_graphics::intoxication_state::*;


define_system! {
    name: ExplorationSystem;
    resources(player: Entity, position_cache: PositionCache);
    fn process_all_entities(&mut self, cs: &mut Components, _dt: Duration, _entities: Entities) {
        let player = *self.player();
        if !(cs.has::<AcceptsUserInput>(player) && cs.has::<Position>(player) && cs.has::<Exploration>(player) && cs.has::<Attributes>(player)) {
            return
        }
        let cache = &*self.position_cache();
        let pos = cs.get::<Position>(player);
        let exploration = cs.get::<Exploration>(player);
        let attrs = cs.get::<Attributes>(player);
        let radius = match IntoxicationState::from_int(attrs.state_of_mind) {
            Exhausted | DeliriumTremens => 4,
            Withdrawal => 5,
            Sober => 6,
            High => 7,
            VeryHigh | Overdosed => 8,
        };
        if radius != exploration.radius {
            cs.set(Exploration{radius: radius}, player);
        }
        for (x, y) in point::points_within_radius(pos, radius) {
            if point::distance(pos, (x, y)) < (radius + 1) as f32 {
                for exploree in cache.entities_on_pos((x, y)) {
                    if cs.has::<Tile>(exploree) && cs.has::<Position>(exploree) {
                        cs.set(Explored, exploree);
                    }
                }
            }
        }
    }
}
