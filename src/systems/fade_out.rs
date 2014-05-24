use components::{FadeOut, FadingOut, Tile};
use components::{Count};
use super::super::Resources;

pub fn system(e: ID,
              ecm: &mut ComponentManager,
              _res: &mut Resources) {
    ensure_components!(ecm, e, FadeOut, Tile);
    let fade_out = ecm.get_fade_out(e);
    let tile = ecm.get_tile(e);
    if !ecm.has_fading_out(e) {
        // replace any existing animation with our fade out
        ecm.remove_color_animation(e);
        ecm.set(e, FadeColor{
                from: tile.color,
                to: fade_out.to,
                duration_s: fade_out.duration_s,
                repetitions: Count(1),
            });
        ecm.set_fading_out(e, FadingOut);
    } else if !ecm.has_color_animation(e) {
        // the animation has ended, finish the fade out
        ecm.remove_tile(e);
        ecm.remove_fade_out(e);
        ecm.remove_fading_out(e);
    }
}
