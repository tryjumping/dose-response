use crate::random::Random;

use rodio::{OutputStreamHandle, Sink};

pub struct Audio {
    pub backgrounds: BackgroundSounds,
    pub background_sound_queue: Sink,
    pub effects: EffectSounds,
    pub sound_effect_queue: [Sink; 3],
    pub rng: Random,
}

impl Audio {
    pub fn new(stream_handle: &OutputStreamHandle) -> Self {
        let background_sound_queue =
            Sink::try_new(&stream_handle).unwrap_or_else(|_| Sink::new_idle().0);
        let sound_effect_queue = [
            Sink::try_new(&stream_handle).unwrap_or_else(|_| Sink::new_idle().0),
            Sink::try_new(&stream_handle).unwrap_or_else(|_| Sink::new_idle().0),
            Sink::try_new(&stream_handle).unwrap_or_else(|_| Sink::new_idle().0),
        ];

        let forrest = {
            let bytes = include_bytes!("../assets/music/AMBForst_Forest (ID 0100)_BSB.ogg");
            std::io::Cursor::new(&bytes[..])
        };

        let cow = {
            let bytes = include_bytes!("../assets/sound/ANMLFarm_Sheep 7 (ID 2349)_BSB.ogg");
            std::io::Cursor::new(&bytes[..])
        };

        let walk_1 = {
            let bytes = include_bytes!("../assets/sound/walk-1.ogg");
            std::io::Cursor::new(&bytes[..])
        };

        let walk_2 = {
            let bytes = include_bytes!("../assets/sound/walk-2.ogg");
            std::io::Cursor::new(&bytes[..])
        };

        let walk_3 = {
            let bytes = include_bytes!("../assets/sound/walk-3.ogg");
            std::io::Cursor::new(&bytes[..])
        };

        let walk_4 = {
            let bytes = include_bytes!("../assets/sound/walk-4.ogg");
            std::io::Cursor::new(&bytes[..])
        };

        let monster_hit = {
            let bytes = include_bytes!("../assets/sound/monster-hit.ogg");
            std::io::Cursor::new(&bytes[..])
        };

        let explosion = {
            let bytes = include_bytes!("../assets/sound/explosion.ogg");
            std::io::Cursor::new(&bytes[..])
        };

        Self {
            backgrounds: BackgroundSounds {
                ambient_forrest: forrest,
                cow: cow,
            },
            effects: EffectSounds {
                walk: [walk_1, walk_2, walk_3, walk_4],
                monster_hit,
                explosion,
            },
            background_sound_queue,
            sound_effect_queue,
            rng: Random::new(),
        }
    }

    pub fn play_sound_effect(&mut self, effect: Effect) {
        use Effect::*;

        let data = match effect {
            Walk => self
                .rng
                .choose_with_fallback(&self.effects.walk, &self.effects.walk[0])
                .clone(),

            MonsterHit => self.effects.monster_hit.clone(),

            Explosion => self.effects.explosion.clone(),
        };

        match rodio::Decoder::new(data) {
            Ok(sound) => {
                let mut all_queues_full = true;
                for sink in self.sound_effect_queue.iter() {
                    if sink.empty() {
                        sink.append(sound);
                        all_queues_full = false;
                        break;
                    }
                }
                if all_queues_full {
                    log::warn!("play_sound_effect: no empty queue found. Skipping playback.");
                }
            }
            Err(error) => {
                log::error!(
                    "play_sound_effect: Error decoding sound: {}. Skipping playback.",
                    error
                );
            }
        }
    }
}

pub struct BackgroundSounds {
    pub ambient_forrest: std::io::Cursor<&'static [u8]>,
    // TODO: replace this with proper other ambient sound effects
    pub cow: std::io::Cursor<&'static [u8]>,
}

impl BackgroundSounds {
    pub fn random(&self, rng: &mut Random) -> std::io::Cursor<&'static [u8]> {
        match rng.range_inclusive(1, 2) {
            1 => self.ambient_forrest.clone(),
            2 => self.cow.clone(),
            unexpected => {
                log::error!(
                    "BackgroundSounds::random: Unexpected random number came up: {}",
                    unexpected
                );
                self.ambient_forrest.clone()
            }
        }
    }
}

pub struct EffectSounds {
    pub walk: [std::io::Cursor<&'static [u8]>; 4],
    pub monster_hit: std::io::Cursor<&'static [u8]>,
    pub explosion: std::io::Cursor<&'static [u8]>,
}

pub enum Effect {
    Walk,
    MonsterHit,
    Explosion,
}
