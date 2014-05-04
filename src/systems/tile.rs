use emhyr::{ComponentManager, ECM, Entity};
use components::{Background, ColorAnimation, Exploration, Explored, Position, Tile};
use engine::Display;
use util::precise_distance;
use color = world::col;


define_system! {
    name: TileSystem;
    components(Position, Tile);
    resources(ecm: ECM, display: Display, player: Entity);
    fn process_entity(&mut self, dt_ms: uint, e: Entity) {
        let player = *self.player();
        let mut ecm = self.ecm();
        let Position{x, y} = ecm.get::<Position>(e);
        let Tile{level, glyph, color} = ecm.get::<Tile>(e);
        let is_visible = if ecm.has::<Position>(player) && ecm.has::<Exploration>(player) {
            let player_pos: Position = ecm.get(player);
            precise_distance((x, y), (player_pos.x, player_pos.y)) <= ecm.get::<Exploration>(player).radius
        } else {
            false
        };
        let shows_in_fog_of_war = ecm.has::<Background>(e) || ecm.has::<Explored>(e);
        let is_explored = ecm.has::<Explored>(e);
        // TODO: get the current value of cheating from GameState
        let cheating = true;
        if is_explored || cheating {
            let bg = if is_visible {
                color::background
            } else {
                color::dim_background
            };
            if is_visible || shows_in_fog_of_war || cheating {
                let final_color = if ecm.has::<ColorAnimation>(e) {
                    ecm.get::<ColorAnimation>(e).color
                } else {
                    color
                };
                self.display().draw_char(level, x, y, glyph, final_color, bg);
            }
        }
    }
}
