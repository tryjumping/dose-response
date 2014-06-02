use components::{Background, ColorAnimation, Exploration, Explored, Position, Tile};
use ecm::{ComponentManager, ECM, Entity};
use engine::Display;
use point;
use color = world::col;


define_system! {
    name: TileSystem;
    components(Position, Tile);
    resources(ecm: ECM, display: Display, player: Entity, cheating: bool);
    fn process_entity(&mut self, dt_ms: uint, e: Entity) {
        let player = *self.player();
        let mut ecm = self.ecm();
        let Position{x, y} = ecm.get::<Position>(e);
        let Tile{level, glyph, color} = ecm.get::<Tile>(e);
        let is_visible = if ecm.has::<Position>(player) && ecm.has::<Exploration>(player) {
            let player_pos: Position = ecm.get(player);
            point::distance((x, y), player_pos) <= ecm.get::<Exploration>(player).radius as f32
        } else {
            false
        };
        let shows_in_fog_of_war = ecm.has::<Background>(e) || ecm.has::<Explored>(e);
        let is_explored = ecm.has::<Explored>(e);
        let cheating = *self.cheating();
        if is_explored || cheating {
            let bg = if is_visible {
                color::background
            } else {
                color::dim_background
            };
            if is_visible || shows_in_fog_of_war || cheating {
                let final_color = if ecm.has::<ColorAnimation>(e) {
                    ecm.get::<ColorAnimation>(e).current.color
                } else {
                    color
                };
                self.display().draw_char(level, x, y, glyph, final_color, bg);
            }
        }
    }
}
