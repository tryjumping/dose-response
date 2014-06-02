use components::{ColorAnimation, ColorAnimationState};
use components::{Count, Infinite, Sec, Forward, Backward};
use ecm::{ComponentManager, ECM, Entity};
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

define_system! {
    name: ColorAnimationSystem;
    components(ColorAnimation);
    resources(ecm: ECM);
    fn process_entity(&mut self, dt_ms: uint, entity: Entity) {
        let mut ecm = self.ecm();
        let animation = ecm.get::<ColorAnimation>(entity);
        let dt = Sec(dt_ms as f32 / 1000.0);

        let mut direction = animation.current.fade_direction;
        let mut repetitions = animation.repetitions;
        let mut elapsed_time = Sec({
            let Sec(dt) = dt;
            let Sec(prev_et) = animation.current.elapsed_time;
            prev_et + dt
        });

        let transition_complete = {
            let Sec(elapsed) = elapsed_time;
            let Sec(duration) = animation.transition_duration;
            elapsed >= duration
        };
        if transition_complete {
            match repetitions {
                Count(0) | Count(1) => {
                    ecm.remove::<ColorAnimation>(entity);
                    return
                }
                Count(n) => repetitions = Count(n - 1),
                Infinite => {},
            }
            elapsed_time = Sec(0.0);
            direction = match direction {
                Forward => Backward,
                Backward => Forward,
            };
        }

        let fade_progress = {
            let Sec(elapsed) = elapsed_time;
            let Sec(duration) = animation.transition_duration;
            elapsed / duration
        };
        let current_color = match direction {
            Forward => fade_color(animation.from, animation.to, fade_progress),
            Backward => fade_color(animation.to, animation.from, fade_progress),
        };

        ecm.set(entity, ColorAnimation{
            repetitions: repetitions,
            current: ColorAnimationState{
                color: current_color,
                fade_direction: direction,
                elapsed_time: elapsed_time,
            },
            .. animation
        })
    }
}
