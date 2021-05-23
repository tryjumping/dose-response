use crate::{random::Random, util};

use std::{convert::TryInto, time::Duration};

use rodio::{OutputStreamHandle, Sink, Source};

type SoundData = std::io::Cursor<&'static [u8]>;

pub struct Audio {
    pub backgrounds: BackgroundSounds,
    pub background_sound_queue: Sink,
    pub effects: EffectSounds,
    pub sound_effect_queue: [Sink; 6],
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
        let sound_effect_queue = [
            stream_handle.map_or_else(empty_sink, new_sink),
            stream_handle.map_or_else(empty_sink, new_sink),
            stream_handle.map_or_else(empty_sink, new_sink),
            stream_handle.map_or_else(empty_sink, new_sink),
            stream_handle.map_or_else(empty_sink, new_sink),
            stream_handle.map_or_else(empty_sink, new_sink),
        ];

        let forrest = SoundData::new(
            &include_bytes!("../assets/music/AMBForst_Forest (ID 0100)_BSB.ogg")[..],
        );

        let cow = SoundData::new(
            &include_bytes!("../assets/sound/ANMLFarm_Sheep 7 (ID 2349)_BSB.ogg")[..],
        );

        let walk_1 = SoundData::new(&include_bytes!("../assets/sound/walk-1.ogg")[..]);

        let walk_2 = SoundData::new(&include_bytes!("../assets/sound/walk-2.ogg")[..]);

        let walk_3 = SoundData::new(&include_bytes!("../assets/sound/walk-3.ogg")[..]);

        let walk_4 = SoundData::new(&include_bytes!("../assets/sound/walk-4.ogg")[..]);

        let monster_hit = SoundData::new(&include_bytes!("../assets/sound/monster-hit.ogg")[..]);

        let monster_moved = SoundData::new(&include_bytes!("../assets/sound/blip.ogg")[..]);

        let explosion = SoundData::new(&include_bytes!("../assets/sound/explosion.ogg")[..]);

        let game_over = SoundData::new(&include_bytes!("../assets/sound/game-over.ogg")[..]);

        Self {
            backgrounds: BackgroundSounds {
                ambient_forrest: forrest,
                cow: cow,
            },
            effects: EffectSounds {
                walk: [walk_1, walk_2, walk_3, walk_4],
                monster_hit,
                monster_moved,
                explosion,
                game_over,
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

    fn data_from_effect(&mut self, effect: Effect) -> SoundData {
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
        }
    }

    pub fn play_mixed_sound_effects(&mut self) {
        use rodio::{decoder::Decoder, source::Empty};
        let mut mixed_sound: Box<dyn Source<Item = i16> + Send> = Box::new(Empty::new());
        while let Some((effect, delay)) = self.sound_effects.pop() {
            let data = self.data_from_effect(effect);
            if let Ok(sound) = Decoder::new(data) {
                mixed_sound = Box::new(mixed_sound.mix(sound.delay(delay)));
            }
        }
        self.play_sound(mixed_sound);
    }

    fn play_sound<S: 'static + Source<Item = i16> + Send>(&mut self, sound: S) {
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

    pub fn set_background_volume(&mut self, volume: f32) {
        let volume = util::clampf(0.0, volume, 1.0);
        self.background_sound_queue.set_volume(volume);
    }

    pub fn set_effects_volume(&mut self, volume: f32) {
        let volume = util::clampf(0.0, volume, 1.0);
        for queue in self.sound_effect_queue.iter_mut() {
            queue.set_volume(volume);
        }
    }
}

pub struct BackgroundSounds {
    pub ambient_forrest: SoundData,
    // TODO: replace this with proper other ambient sound effects
    pub cow: SoundData,
}

impl BackgroundSounds {
    pub fn random(&self, rng: &mut Random) -> SoundData {
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
    pub walk: [SoundData; 4],
    pub monster_hit: SoundData,
    pub monster_moved: SoundData,
    pub explosion: SoundData,
    pub game_over: SoundData,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Effect {
    Walk,
    MonsterHit,
    MonsterMoved,
    Explosion,
    GameOver,
}
