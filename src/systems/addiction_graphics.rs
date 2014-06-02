use std::cmp::max;

use ecm::{ComponentManager, ECM, Entity};

use components::{Attributes, Background, ColorAnimation, ColorAnimationState, Position};
use components::{Infinite, Sec, Forward};
use engine::Display;
use self::intoxication_state::*;
use world::col;
use entity_util;

pub mod intoxication_state {
    #[deriving(PartialEq)]
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

define_system! {
    name: AddictionGraphicsSystem;
    resources(ecm: ECM, display: Display, player: Entity);
    fn process_all_entities(&mut self, _dt_ms: uint, mut _entities: &mut Iterator<Entity>) {
        let mut ecm = &mut *self.ecm();
        let mut display = self.display();
        let player = *self.player();
        if !ecm.has::<Attributes>(player) {return}

        let som = ecm.get::<Attributes>(player).state_of_mind;
        match IntoxicationState::from_int(som) {
            Exhausted | DeliriumTremens | Withdrawal => {
                let fade = max((som as u8) * 5 + 50, 50);
                display.fade(fade, col::background);
                for e in ecm.iter() {
                    if ecm.has::<Background>(e) && ecm.has::<ColorAnimation>(e) {
                        ecm.remove::<ColorAnimation>(e);
                    }
                }
            }
            Sober => {
                for e in ecm.iter() {
                    if ecm.has::<Background>(e) && ecm.has::<ColorAnimation>(e) {
                        ecm.remove::<ColorAnimation>(e);
                    }
                }
            }
            High | VeryHigh | Overdosed => {
                for e in ecm.iter() {
                    if !ecm.has_entity(e) || !ecm.has::<Position>(e) {continue}
                    let pos: Position = ecm.get(e);
                    if !ecm.has::<ColorAnimation>(e) && ecm.has::<Background>(e) {
                        entity_util::set_color_animation_loop(
                            ecm, e, col::high, col::high_to, Infinite,
                            Sec(0.7 + (((pos.x * pos.y) % 100) as f32) / 200.0));
                    }
                }
            }
        }
    }
}
