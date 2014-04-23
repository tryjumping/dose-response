use emhyr::{ComponentManager, ECM, Entity};
use components::*;
use engine::Display;
use util::precise_distance;
use world::col;

pub fn system(e: Entity,
              ecm: &mut ECM,
              display: &mut Display) {
    fail!("TODO");
    // ensure_components!(ecm, e, Position, Tile);
    // let player = res.player;
    // let Position{x, y} = ecm.get::<Position>(e);
    // let Tile{level, glyph, color} = ecm.get::<Tile>(e);
    // let is_visible = if ecm.has::<Position>(player) && ecm.has::<Exploration>(player) {
    //     let player_pos: Position = ecm.get(player);
    //     precise_distance((x, y), (player_pos.x, player_pos.y)) <= ecm.get::<Exploration>(res.player).radius
    // } else {
    //     false
    // };
    // let shows_in_fog_of_war = ecm.has::<Background>(e) || ecm.has::<Explored>(e);
    // let is_explored = ecm.has::<Explored>(e);
    // if is_explored || res.cheating {
    //     let bg = if is_visible {
    //         col::background
    //     } else {
    //         col::dim_background
    //     };
    //     if is_visible || shows_in_fog_of_war || res.cheating {
    //         let final_color = if ecm.has::<ColorAnimation>(e) {
    //             ecm.get::<ColorAnimation>(e).color
    //         } else {
    //             color
    //         };
    //         display.draw_char(level, x, y, glyph, final_color, bg);
    //     }
    // }
}
