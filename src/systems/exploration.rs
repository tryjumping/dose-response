use components::{AcceptsUserInput, Position, Exploration, Attributes, Explored, Tile};
use ecm::{ComponentManager, ECM, Entity};
use point;
use super::addiction_graphics::intoxication_state::*;


define_system! {
    name: ExplorationSystem;
    resources(ecm: ECM, player: Entity);
    fn process_all_entities(&mut self, _dt_ms: uint, mut _entities: &mut Iterator<Entity>) {
        let mut ecm = self.ecm();
        let player = *self.player();
        if !(ecm.has::<AcceptsUserInput>(player) && ecm.has::<Position>(player) && ecm.has::<Exploration>(player) && ecm.has::<Attributes>(player)) {
            return
        }
        let pos = ecm.get::<Position>(player);
        let exploration = ecm.get::<Exploration>(player);
        let attrs = ecm.get::<Attributes>(player);
        let radius = match IntoxicationState::from_int(attrs.state_of_mind) {
            Exhausted | DeliriumTremens => 4,
            Withdrawal => 5,
            Sober => 6,
            High => 7,
            VeryHigh | Overdosed => 8,
        };
        if radius != exploration.radius {
            ecm.set(player, Exploration{radius: radius});
        }
        for (x, y) in point::points_within_radius(pos, radius) {
            if point::distance(pos, (x, y)) <= radius as f32 {
                for exploree in ecm.entities_on_pos((x, y)) {
                    if ecm.has::<Tile>(exploree) && ecm.has::<Position>(exploree) {
                        ecm.set(exploree, Explored);
                    }
                }
            }
        }
    }
}
