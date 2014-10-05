use std::time::Duration;
use std::cmp::max;

use emhyr::{Components, Entity, Entities};

use components::{Attributes, Background, ColorAnimation, Position};
use components::{Infinite};
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
                1...5   => DeliriumTremens,
                6...15  => Withdrawal,
                16...20 => Sober,
                21...80 => High,
                81...99 => VeryHigh,
                _ => Overdosed,
            }
        }
    }
}

define_system! {
    name: AddictionGraphicsSystem;
    resources(display: Display, player: Entity);
    fn process_all_entities(&mut self, cs: &mut Components, _dt: Duration, entities: Entities) {
        let mut display = self.display();
        let player = *self.player();
        if !cs.has::<Attributes>(player) {return}

        let som = cs.get::<Attributes>(player).state_of_mind;
        match IntoxicationState::from_int(som) {
            Exhausted | DeliriumTremens | Withdrawal => {
                let fade = max((som as u8) * 5 + 50, 50);
                display.fade(fade, col::background);
                let mut entities = entities;
                for e in entities {
                    if cs.has::<Background>(e) && cs.has::<ColorAnimation>(e) {
                        cs.unset::<ColorAnimation>(e);
                    }
                }
            }
            Sober => {
                let mut entities = entities;
                for e in entities {
                    if cs.has::<Background>(e) && cs.has::<ColorAnimation>(e) {
                        cs.unset::<ColorAnimation>(e);
                    }
                }
            }
            High | VeryHigh | Overdosed => {
                let mut entities = entities;
                for e in entities {
                    if !cs.has::<Position>(e) {continue}
                    let pos = cs.get::<Position>(e);
                    if !cs.has::<ColorAnimation>(e) && cs.has::<Background>(e) {
                        entity_util::set_color_animation_loop(
                            cs, e, col::high, col::high_to, Infinite,
                            Duration::milliseconds(700 + (((pos.x * pos.y) % 100) as i64) * 5));
                    }
                }
            }
        }
    }
}
