use std::cmp::max;

use emhyr::{ComponentManager, ECM, Entity};

use components::*;
use engine::Display;
use self::intoxication_state::*;
use world::col;

pub mod intoxication_state {
    #[deriving(Eq)]
    pub enum IntoxicationState {
        Exhausted,
        DeliriumTremens,
        Withdrawal,
        Sober,
        High,
        VeryHigh,
        Overdosed,
    }

    impl IntoxicationState {
        pub fn from_int(value: int) -> IntoxicationState {
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

pub fn system(ecm: &mut ECM,
              display: &mut Display) {
    fail!("TODO");
    // ensure_components!(ecm, res.player, Attributes);
    // let som = ecm.get::<Attributes>(res.player).state_of_mind;
    // match IntoxicationState::from_int(som) {
    //     Exhausted | DeliriumTremens | Withdrawal => {
    //         let fade = max((som as u8) * 5 + 50, 50);
    //         display.fade(fade, col::background);
    //         for e in ecm.iter() {
    //             if ecm.has::<Background>(e) && ecm.has::<FadeColor>(e) {
    //                 ecm.remove::<FadeColor>(e);
    //             }
    //         }
    //     }
    //     Sober => {
    //         for e in ecm.iter() {
    //             if ecm.has::<Background>(e) && ecm.has::<FadeColor>(e) {
    //                 ecm.remove::<FadeColor>(e);
    //             }
    //         }
    //     }
    //     High | VeryHigh | Overdosed => {
    //         for e in ecm.iter() {
    //             if !ecm.has_entity(e) || !ecm.has::<Position>(e) {continue}
    //             let pos: Position = ecm.get(e);
    //             if !ecm.has::<ColorAnimation>(e) && ecm.has::<Background>(e) {
    //                 ecm.set(e, FadeColor{
    //                         from: col::high,
    //                         to: col::high_to,
    //                         repetitions: Infinite,
    //                         duration_s: 0.7 + (((pos.x * pos.y) % 100) as f32) / 200.0,
    //                     });
    //             }
    //         }
    //     }
    // }
}
