use std::time::Duration;

use components::{Background, ColorAnimation, Exploration, Explored, Position, Tile};
use emhyr::{Components, Entity};
use engine::Display;
use point;
use world::col as color;


define_system! {
    name: TileSystem;
    components(Position, Tile);
    resources(display: Display, player: Entity, cheating: bool);
    fn process_entity(&mut self, cs: &mut Components, _dt: Duration, e: Entity) {
        let player = *self.player();
        let Position{x, y} = cs.get::<Position>(e);
        let Tile{level, glyph, color} = cs.get::<Tile>(e);
        let is_visible = if cs.has::<Position>(player) && cs.has::<Exploration>(player) {
            let player_pos: Position = cs.get(player);
            point::distance((x, y), player_pos) <= cs.get::<Exploration>(player).radius as f32
        } else {
            false
        };
        let shows_in_fog_of_war = cs.has::<Background>(e) || cs.has::<Explored>(e);
        let is_explored = cs.has::<Explored>(e);
        let cheating = *self.cheating();
        if is_explored || cheating {
            let bg = if is_visible {
                color::background
            } else {
                color::dim_background
            };
            if is_visible || shows_in_fog_of_war || cheating {
                let final_color = if cs.has::<ColorAnimation>(e) {
                    cs.get::<ColorAnimation>(e).current.color
                } else {
                    color
                };
                self.display().draw_char(level, x, y, glyph, final_color, bg);
            }
        }
    }
}
