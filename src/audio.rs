use crate::random::Random;

use std::{convert::TryInto, time::Duration};

use rodio::{OutputStreamHandle, Sink, Source};

type SoundData = std::io::Cursor<&'static [u8]>;

pub struct Audio {
    pub backgrounds: BackgroundSounds,
    pub background_sound_queue: Sink,
    pub effects: EffectSounds,
    pub sound_effect_queue: [Sink; 3],
    pub rng: Random,
    sound_effects: Vec<Effect>,
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

    // TODO: require a delay here?
    pub fn mix_sound_effect(&mut self, effect: Effect) {
        self.sound_effects.push(effect);
    }

    fn random_delay(&mut self) -> Duration {
        Duration::from_millis(self.rng.range_inclusive(1, 50).try_into().unwrap_or(0))
    }

    fn data_from_effect(&mut self, effect: Effect) -> (SoundData, Duration) {
        use Effect::*;
        match effect {
            Walk => {
                let data = self
                    .rng
                    .choose_with_fallback(&self.effects.walk, &self.effects.walk[0])
                    .clone();
                (data, Duration::from_secs(0))
            }
            MonsterHit => (self.effects.monster_hit.clone(), self.random_delay()),
            MonsterMoved => (self.effects.monster_moved.clone(), self.random_delay()),
            Explosion => (self.effects.explosion.clone(), self.random_delay()),
            GameOver => (self.effects.game_over.clone(), self.random_delay()),
        }
    }

    pub fn play_mixed_sound_effects(&mut self) {
        if let Some(effect) = self.sound_effects.pop() {
            let (data, delay) = self.data_from_effect(effect);

            if let Ok(sound) = rodio::Decoder::new(data) {
                let mut sound: Box<dyn Source<Item = i16> + Send> = Box::new(sound.delay(delay));
                while let Some(effect) = self.sound_effects.pop() {
                    let (data, delay) = self.data_from_effect(effect);
                    if let Ok(s) = rodio::Decoder::new(data) {
                        sound = Box::new(sound.mix(s.delay(delay)));
                    }
                }
                self.play_sound(sound);
            }
        }
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
