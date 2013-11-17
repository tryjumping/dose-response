use components::*;
use super::super::Resources;
use engine::Display;
use super::exploration::precise_distance;
use world::col;

pub fn system(e: ID,
              ecm: &mut ComponentManager,
              res: &mut Resources,
              display: &mut Display) {
    ensure_components!(ecm, e, Position, Tile);
    let player = res.player_id;
    let Position{x, y} = ecm.get_position(e);
    let Tile{level, glyph, color} = ecm.get_tile(e);
    let is_visible = if ecm.has_position(player) && ecm.has_exploration(player) {
        let player_pos = ecm.get_position(res.player_id);
        precise_distance((x, y), (player_pos.x, player_pos.y)) <= ecm.get_exploration(res.player_id).radius
    } else {
        false
    };
    let shows_in_fog_of_war = ecm.has_background(e) || ecm.has_dose(e);
    // TODO: fix exploration
    //let is_explored = res.map.is_explored((x, y));
    let is_explored = true;
    if is_explored || res.cheating {
        let bg = if is_visible {
            col::background
        } else {
            col::dim_background
        };
        if is_visible || shows_in_fog_of_war || res.cheating {
            let final_color = if ecm.has_color_animation(e) {
                ecm.get_color_animation(e).color
            } else {
                color
            };
            display.draw_char(level, x, y, glyph, final_color, bg);
        }
    }
}
