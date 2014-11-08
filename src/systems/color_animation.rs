use std::time::Duration;
use components::{ColorAnimation, ColorAnimationState};
use components::{Count, Infinite, Forward, Backward};
use emhyr::{Components, Entity};
use engine::Color;


fn fade_color(from: Color, to: Color, progress: f32) -> Color {
    #![allow(unused_parens)]
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

define_system! {
    name: ColorAnimationSystem;
    components(ColorAnimation);
    resources(player: Entity);
    fn process_entity(&mut self, cs: &mut Components, dt: Duration, entity: Entity) {
        let animation = cs.get::<ColorAnimation>(entity);
        let mut direction = animation.current.fade_direction;
        let mut repetitions = animation.repetitions;
        let mut elapsed_time = animation.current.elapsed_time + dt;

        let transition_complete = elapsed_time >= animation.transition_duration;
        if transition_complete {
            match repetitions {
                Count(0) | Count(1) => {
                    cs.unset::<ColorAnimation>(entity);
                    return
                }
                Count(n) => repetitions = Count(n - 1),
                Infinite => {},
            }
            elapsed_time = Duration::milliseconds(0);
            direction = match direction {
                Forward => Backward,
                Backward => Forward,
            };
        }

        let fade_progress = elapsed_time.num_milliseconds() as f32 / animation.transition_duration.num_milliseconds() as f32;
        let current_color = match direction {
            Forward => fade_color(animation.from, animation.to, fade_progress),
            Backward => fade_color(animation.to, animation.from, fade_progress),
        };

        cs.set(ColorAnimation{
            repetitions: repetitions,
            current: ColorAnimationState{
                color: current_color,
                fade_direction: direction,
                elapsed_time: elapsed_time,
            },
            .. animation
        }, entity);
    }
}
