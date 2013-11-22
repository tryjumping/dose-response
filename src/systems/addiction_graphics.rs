use std::num::max;

use components::*;
use super::super::Resources;
use engine::{Color, Display};


pub fn system(ecm: &mut ComponentManager,
              res: &mut Resources,
              display: &mut Display) {
    ensure_components!(ecm, res.player_id, Attributes);
    let som = ecm.get_attributes(res.player_id).state_of_mind;
    if som <= 20 {
        let fade = max((som as u8) * 5 + 50, 50);
        display.fade(fade, Color{r: 0, g: 0, b: 0});
    }
}
