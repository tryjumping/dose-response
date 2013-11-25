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
    } else if som >= 40 {
        for e in ecm.iter() {
            if !ecm.has_color_animation(e) && ecm.has_background(e) {
                // TODO: unsafe, check for the components first!
                let pos = ecm.get_position(e);
                let col = match som > 70 {
                    true => Color{r: 219, g: 0, b: 40},
                    false => ecm.get_tile(e).color,
                };
                let high_col = Color{r: 58, g: 217, b: 183};
                //let high_col = Color{r: 255, g: 255, b: 25};
                //let high_col = Color{r: col.r+128, g: col.g+128, b: col.b+128};
                ecm.set_fade_color(e, FadeColor{
                        // from: Color{r: col.r + 20, g: col.g - 20, b: col.b + 20},
                        // to: Color{r: col.r - 20, g: col.g + 20, b: col.b - 20},
                        // from: Color{r: col.g, g: col.b, b: col.r},
                        // to: Color{r: col.b, g: col.r, b: col.g},
                        // from: Color{r: 58, g: 217, b: 183},
                        // to: Color{r: 219, g: 0, b: 40},
                        from: high_col,
                        to: Color{r: col.r - high_col.r, g: col.g - high_col.g, b: col.b - high_col.b},
                        repetitions: Infinite,
                        duration_s: 0.7 + (((pos.x * pos.y) % 100) as float) / 200.0,
                    });
            }
        }
    } else {
        for e in ecm.iter() {
            if ecm.has_background(e) && ecm.has_fade_color(e) {
                ecm.remove_fade_color(e);
            }
        }
    }
}
