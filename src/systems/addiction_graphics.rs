use std::num::max;

use components::*;
use super::super::Resources;
use engine::Display;
use self::intoxication_states::*;
use world::col;

pub mod intoxication_states {
    #[deriving(Eq)]
    pub enum IntoxicationStates {
        Exhausted,
        DeliriumTremens,
        Withdrawal,
        Sober,
        High,
        VeryHigh,
        Overdosed,
    }

    impl IntoxicationStates {
        pub fn from_int(value: int) -> IntoxicationStates {
            match value {
                val if val <= 0 => Exhausted,
                1..5   => DeliriumTremens,
                6..15  => Withdrawal,
                16..20 => Sober,
                21..80 => High,
                81..99 => VeryHigh,
                _ => Overdosed,
            }
        }
    }
}

pub fn system(ecm: &mut ComponentManager,
              res: &mut Resources,
              display: &mut Display) {
    ensure_components!(ecm, res.player_id, Attributes);
    let som = ecm.get_attributes(res.player_id).state_of_mind;
    match IntoxicationStates::from_int(som) {
        Exhausted | DeliriumTremens | Withdrawal => {
            let fade = max((som as u8) * 5 + 50, 50);
            display.fade(fade, col::background);
            for e in ecm.iter() {
                if ecm.has_background(e) && ecm.has_fade_color(e) {
                    ecm.remove_fade_color(e);
                }
            }
        }
        Sober => {
            for e in ecm.iter() {
                if ecm.has_background(e) && ecm.has_fade_color(e) {
                    ecm.remove_fade_color(e);
                }
            }
        }
        High | VeryHigh | Overdosed => {
            for e in ecm.iter() {
                if !ecm.has_entity(e) || !ecm.has_position(e) {continue}
                let pos = ecm.get_position(e);
                if !ecm.has_color_animation(e) && ecm.has_background(e) {
                    ecm.set_fade_color(e, FadeColor{
                            from: col::high,
                            to: col::high_to,
                            repetitions: Infinite,
                            duration_s: 0.7 + (((pos.x * pos.y) % 100) as f32) / 200.0,
                        });
                }
            }
        }
    }
}
