use crate::random::Random;

use rodio::{OutputStreamHandle, Sink};

pub struct Audio {
    pub backgrounds: BackgroundSounds,
    pub background_sound_queue: Sink,
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

        Self {
            backgrounds: BackgroundSounds {
                ambient_forrest: forrest,
                cow: cow,
            },
            background_sound_queue,
            sound_effect_queue,
            rng: Random::new(),
        }
    }

    pub fn play_sound_effect(&self, effect: Effect) {
        // TODO: use the actual sound effects rather than hardcoding the backgrounds.cow sound
        match rodio::Decoder::new(self.backgrounds.cow.clone()) {
            Ok(sound) => {
                // TODO Mix the sounds together. `queue.append` always waits for the sound to finish
                // I *think* we'll need to add multiple sinks here and pick the one that's empty
                if self.sound_effect_queue[0].empty() {
                    self.sound_effect_queue[0].append(sound);
                } else if self.sound_effect_queue[1].empty() {
                    self.sound_effect_queue[1].append(sound);
                } else if self.sound_effect_queue[2].empty() {
                    self.sound_effect_queue[2].append(sound);
                } else {
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

pub enum Effect {
    Bump,
    Move,
    EatFood,
    UseDose,
}
