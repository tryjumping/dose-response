use crate::random::Random;

use rodio::{OutputStreamHandle, Sink};

pub struct Audio {
    pub background_sounds: BackgroundSounds,
    pub background_queue: Sink,
}

impl Audio {
    pub fn new(stream_handle: &OutputStreamHandle) -> Self {
        let sink = Sink::try_new(&stream_handle).unwrap_or_else(|_| Sink::new_idle().0);

        let music = include_bytes!("../assets/music/AMBForst_Forest (ID 0100)_BSB.ogg");
        let music = std::io::Cursor::new(&music[..]);

        // let sound = include_bytes!("../assets/sound/ANMLFarm_Sheep 7 (ID 2349)_BSB.ogg");
        // let sound = std::io::Cursor::new(&sound[..]);
        // let sound = Decoder::new(sound).unwrap();
        // let effect_sink = Sink::try_new(&stream_handle).unwrap();
        // background_sink.append(music.repeat_infinite());
        // effect_sink.append(
        //     sound
        //         .delay(std::time::Duration::new(3, 0))
        //         .repeat_infinite(),
        // );

        Self {
            background_sounds: BackgroundSounds {
                ambient_forrest: music,
            },
            background_queue: sink,
        }
    }
}

pub struct BackgroundSounds {
    pub ambient_forrest: std::io::Cursor<&'static [u8]>,
}

impl BackgroundSounds {
    pub fn random(&self, _rng: &mut Random) -> std::io::Cursor<&'static [u8]> {
        self.ambient_forrest.clone()
    }
}
