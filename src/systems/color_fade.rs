use components::*;
use engine::Color;
use super::super::Resources;

fn fade_color(from: Color, to: Color, progress: float) -> Color {
    if progress <= 0f32 {
        return from;
    } else if progress >= 1f32 {
        return to;
    };
    let dr = ((to.r as float - from.r as float) * progress);
    let dg = ((to.g as float - from.g as float) * progress);
    let db = ((to.b as float - from.b as float) * progress);
    Color{
        r: (from.r as float + dr) as u8,
        g: (from.g as float + dg) as u8,
        b: (from.b as float + db) as u8,
    }
}

pub fn system(e: ID,
              ecm: &mut ComponentManager,
              _res: &mut Resources,
              dt_s: float) {
    if !ecm.has_entity(e) {return}
    if !ecm.has_fade_color(e) {
        if ecm.has_color_animation(e) {
            // Removing the `FadeColor` component should stop the animation
            ecm.remove_color_animation(e);
        }
        return
    }
    let fade = ecm.get_fade_color(e);
    let mut anim = if ecm.has_color_animation(e) {
        ecm.get_color_animation(e)
    } else {
        ColorAnimation{
            color: fade.from,
            progress: 0f32,
            forward: true,
        }
    };
    let step = dt_s / fade.duration_s;
    anim.progress += step;
    if anim.progress >= 1f32 {
        anim.progress = 0f32;
        anim.forward = !anim.forward;
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
    };
    if anim.forward {
        anim.color = fade_color(fade.from, fade.to, anim.progress);
    } else {
        anim.color = fade_color(fade.to, fade.from, anim.progress);
    }
    ecm.set_color_animation(e, anim);
}
