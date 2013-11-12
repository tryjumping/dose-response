use components::*;
use engine::Color;
use super::super::Resources;

pub fn system(e: ID,
              ecm: &mut ComponentManager,
              _res: &mut Resources,
              dt_s: float) {
    ensure_components!(ecm, e, FadeColor);
    let mut fade = ecm.get_fade_color(e);
    let mut anim = if ecm.has_color_animation(e) {
        ecm.get_color_animation(e)
    } else {
        ColorAnimation{
            color: fade.from,
            progress: 0f,
        }
    };
    let step = dt_s / fade.duration_s;
    if anim.progress == 1f {
        anim.progress = 0f;
        ecm.set_fade_color(e, FadeColor{
                to: fade.from,
                from: fade.to,
                .. fade});
        fade = ecm.get_fade_color(e);
    }
    let mut progress = anim.progress + step;
    if progress >= 1f {
        progress = 1f;
        anim.color = fade.to;
        match fade.repetitions {
            Count(n) if n > 1 => {
                ecm.set_fade_color(e, FadeColor{repetitions: Count(n-1), .. fade});
            }
            Count(_) => {
                ecm.remove_fade_color(e);
                ecm.remove_color_animation(e);
                return;
            }
            Infinite => {}
        }
    } else {
        let dr = ((fade.to.r as float - fade.from.r as float) * progress);
        let dg = ((fade.to.g as float - fade.from.g as float) * progress);
        let db = ((fade.to.b as float - fade.from.b as float) * progress);
        anim.color = Color{
            r: (fade.from.r as float + dr) as u8,
            g: (fade.from.g as float + dg) as u8,
            b: (fade.from.b as float + db) as u8,
        }
    }
    anim.progress = progress;
    ecm.set_color_animation(e, anim);
}
