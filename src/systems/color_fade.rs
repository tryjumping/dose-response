use emhyr::{ComponentManager, ECM, Entity};
use components::{ColorAnimation, FadeColor};
use components::{Count, Infinite};
use engine::Color;

fn fade_color(from: Color, to: Color, progress: f32) -> Color {
    if progress <= 0f32 {
        return from;
    } else if progress >= 1f32 {
        return to;
    };
    let dr = ((to.r as f32 - from.r as f32) * progress);
    let dg = ((to.g as f32 - from.g as f32) * progress);
    let db = ((to.b as f32 - from.b as f32) * progress);
    Color{
        r: (from.r as f32 + dr) as u8,
        g: (from.g as f32 + dg) as u8,
        b: (from.b as f32 + db) as u8,
    }
}

pub fn system(e: Entity,
              ecm: &mut ECM,
              dt_s: f32) {
    if !ecm.has_entity(e) {return}
    if !ecm.has::<FadeColor>(e) {
        if ecm.has::<ColorAnimation>(e) {
            // Removing the `FadeColor` component should stop the animation
            ecm.remove::<ColorAnimation>(e);
        }
        return
    }
    let fade: FadeColor = ecm.get(e);
    let mut anim: ColorAnimation = if ecm.has::<ColorAnimation>(e) {
        ecm.get(e)
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
                ecm.set(e, FadeColor{repetitions: Count(n-1), .. fade});
            }
            Count(_) => {
                ecm.remove::<FadeColor>(e);
                ecm.remove::<ColorAnimation>(e);
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
    ecm.set(e, anim);
}
