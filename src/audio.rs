use crate::{random::Random, util};

use std::{convert::TryInto, time::Duration};

use rodio::{
    source::{Buffered, Empty},
    Decoder, OutputStreamHandle, Sink, Source,
};

type SoundData = std::io::Cursor<&'static [u8]>;
type Sound = Option<Buffered<Decoder<SoundData>>>;

pub struct Audio {
    pub backgrounds: BackgroundSounds,
    pub background_sound_queue: Sink,
    pub effects: EffectSounds,
    pub sound_effect_queue: [Sink; 2],
    pub rng: Random,
    sound_effects: Vec<(Effect, Duration)>,
}

impl Audio {
    pub fn new(stream_handle: Option<&OutputStreamHandle>) -> Self {
        fn empty_sink() -> Sink {
            Sink::new_idle().0
        }

        fn new_sink(handle: &OutputStreamHandle) -> Sink {
            match Sink::try_new(handle) {
                Ok(sink) => sink,
                Err(e) => {
                    log::error!("Couldn't create sink: {:?}. Falling back to empty one", e);
                    empty_sink()
                }
            }
        }

        let background_sound_queue = stream_handle.map_or_else(empty_sink, new_sink);
        background_sound_queue.pause();

        let sound_effect_queue = [
            stream_handle.map_or_else(empty_sink, new_sink),
            stream_handle.map_or_else(empty_sink, new_sink),
        ];

        let walk_1 = load_sound(&include_bytes!("../assets/sound/walk-1.ogg")[..]);

        let walk_2 = load_sound(&include_bytes!("../assets/sound/walk-2.ogg")[..]);

        let walk_3 = load_sound(&include_bytes!("../assets/sound/walk-3.ogg")[..]);

        let walk_4 = load_sound(&include_bytes!("../assets/sound/walk-4.ogg")[..]);

        let monster_hit = load_sound(&include_bytes!("../assets/sound/monster-hit.ogg")[..]);

        let monster_moved = load_sound(&include_bytes!("../assets/sound/blip.ogg")[..]);

        let explosion = load_sound(&include_bytes!("../assets/sound/explosion.ogg")[..]);

        let game_over = load_sound(&include_bytes!("../assets/sound/game-over.ogg")[..]);

        let click = load_sound(&include_bytes!("../assets/sound/click.ogg")[..]);

        Self {
            backgrounds: BackgroundSounds {
                // Credits: Exit Exit by P C III (CC-BY)
                // https://freemusicarchive.org/music/P_C_III
                exit_exit: load_sound(
                    &include_bytes!("../assets/music/P C III - Exit Exit.ogg")[..],
                ),
                //https://freemusicarchive.org/music/P_C_III/earth2earth/earth2earth_1392
                // Credits: earth2earth by P C III (CC-BY)
                // https://freemusicarchive.org/music/P_C_III
                family_breaks: load_sound(
                    &include_bytes!("../assets/music/P C III - The Family Breaks.ogg")[..],
                ),
                // https://freemusicarchive.org/music/P_C_III/The_Family_Breaks/The_Family_Breaks_1795
                // Credit: The Family Breaks by P C III (CC-BY)
                // https://freemusicarchive.org/music/P_C_III
                earth2earth: load_sound(
                    &include_bytes!("../assets/music/P C III - earth2earth.ogg")[..],
                ),
            },
            effects: EffectSounds {
                walk: [walk_1, walk_2, walk_3, walk_4],
                monster_hit,
                monster_moved,
                explosion,
                game_over,
                click,
            },
            background_sound_queue,
            sound_effect_queue,
            rng: Random::new(),
            sound_effects: vec![],
        }
    }

    pub fn mix_sound_effect(&mut self, effect: Effect, delay: Duration) {
        self.sound_effects.push((effect, delay));
    }

    pub fn random_delay(&mut self) -> Duration {
        Duration::from_millis(self.rng.range_inclusive(1, 50).try_into().unwrap_or(0))
    }

    fn data_from_effect(&mut self, effect: Effect) -> Sound {
        use Effect::*;
        match effect {
            Walk => self
                .rng
                .choose_with_fallback(&self.effects.walk, &self.effects.walk[0])
                .clone(),
            MonsterHit => self.effects.monster_hit.clone(),
            MonsterMoved => self.effects.monster_moved.clone(),
            Explosion => self.effects.explosion.clone(),
            GameOver => self.effects.game_over.clone(),
            Click => self.effects.click.clone(),
        }
    }

    pub fn play_mixed_sound_effects(&mut self) {
        let mut mixed_sound: Box<dyn Source<Item = i16> + Send> = Box::new(Empty::new());
        while let Some((effect, delay)) = self.sound_effects.pop() {
            if let Some(sound) = self.data_from_effect(effect) {
                mixed_sound = Box::new(mixed_sound.mix(sound.delay(delay)));
            }
        }
        self.play_sound(mixed_sound);
    }

    fn play_sound<S: 'static + Source<Item = i16> + Send>(&mut self, sound: S) {
        for sink in &self.sound_effect_queue {
            if sink.empty() {
                sink.append(sound);
                break;
            }
        }
    }

    pub fn set_background_volume(&mut self, volume: f32) {
        let volume = util::clampf(0.0, volume, 1.0);
        self.background_sound_queue.set_volume(volume);
    }

    pub fn set_effects_volume(&mut self, volume: f32) {
        let volume = util::clampf(0.0, volume, 1.0);
        for queue in &mut self.sound_effect_queue {
            queue.set_volume(volume);
        }
    }
}

fn load_sound(input: &'static [u8]) -> Sound {
    Decoder::new(SoundData::new(input))
        .map(Decoder::buffered)
        .ok()
}

pub struct BackgroundSounds {
    pub exit_exit: Sound,
    pub family_breaks: Sound,
    pub earth2earth: Sound,
}

impl BackgroundSounds {
    pub fn random(&self, rng: &mut Random) -> Sound {
        match rng.range_inclusive(1, 3) {
            1 => self.exit_exit.clone(),
            2 => self.family_breaks.clone(),
            3 => self.earth2earth.clone(),
            unexpected => {
                log::error!(
                    "BackgroundSounds::random: Unexpected random number came up: {}",
                    unexpected
                );
                self.exit_exit.clone()
            }
        }
    }
}

pub struct EffectSounds {
    pub walk: [Sound; 4],
    pub monster_hit: Sound,
    pub monster_moved: Sound,
    pub explosion: Sound,
    pub game_over: Sound,
    pub click: Sound,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Effect {
    Walk,
    MonsterHit,
    MonsterMoved,
    Explosion,
    GameOver,
    Click,
}
