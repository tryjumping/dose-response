use crate::random::Random;

use rodio::{OutputStreamHandle, Sink};

pub struct Audio {
    pub background_sounds: BackgroundSounds,
    pub background_queue: Sink,
    pub rng: Random,
}

impl Audio {
    pub fn new(stream_handle: &OutputStreamHandle) -> Self {
        let sink = Sink::try_new(&stream_handle).unwrap_or_else(|_| Sink::new_idle().0);

        let forrest = {
            let bytes = include_bytes!("../assets/music/AMBForst_Forest (ID 0100)_BSB.ogg");
            std::io::Cursor::new(&bytes[..])
        };

        let cow = {
            let bytes = include_bytes!("../assets/sound/ANMLFarm_Sheep 7 (ID 2349)_BSB.ogg");
            std::io::Cursor::new(&bytes[..])
        };

        Self {
            background_sounds: BackgroundSounds {
                ambient_forrest: forrest,
                cow: cow,
            },
            background_queue: sink,
            rng: Random::new(),
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
