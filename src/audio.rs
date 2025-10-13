use crate::{random::Random, util};

use std::time::Duration;

use rodio::{Decoder, OutputStream, OutputStreamBuilder, Sink, Source};

type Sound = std::io::Cursor<&'static [u8]>;

fn empty_sink() -> Sink {
    Sink::new().0
}

pub struct Audio {
    pub backgrounds: BackgroundSounds,
    pub background_sound_queue: Sink,
    effects: EffectSounds,
    sound_effect_muted: bool,
    output_stream: Option<OutputStream>,
    /// Internal random source
    rng: Random,
}

impl Audio {
    pub fn without_backend() -> Self {
        Self::from_output_stream(None)
    }

    pub fn new() -> Self {
        let output_stream = OutputStreamBuilder::open_default_stream().ok();
        Self::from_output_stream(output_stream)
    }

    fn from_output_stream(output_stream: Option<OutputStream>) -> Self {
        log::info!("Setting up the audio stream.");

        let rng = Random::from_seed(util::random_seed());

        let mixer = output_stream.as_ref().map(OutputStream::mixer);

        let background_sound_queue = mixer.map_or_else(empty_sink, Sink::connect_new);

        // Start paused, let the game code control when audio starts playing.
        background_sound_queue.pause();

        let walk_1 = Sound::new(&include_bytes!("../assets/sound/walk-1.ogg")[..]);
        let walk_2 = Sound::new(&include_bytes!("../assets/sound/walk-2.ogg")[..]);
        let walk_3 = Sound::new(&include_bytes!("../assets/sound/walk-3.ogg")[..]);
        let walk_4 = Sound::new(&include_bytes!("../assets/sound/walk-4.ogg")[..]);
        let monster_hit = Sound::new(&include_bytes!("../assets/sound/monster-hit.ogg")[..]);
        let monster_moved = Sound::new(&include_bytes!("../assets/sound/blip.ogg")[..]);
        let explosion = Sound::new(&include_bytes!("../assets/sound/explosion.ogg")[..]);
        let player_hit = Sound::new(&include_bytes!("../assets/sound/player-hit.ogg")[..]);
        let game_over = Sound::new(&include_bytes!("../assets/sound/game-over.ogg")[..]);
        let click = Sound::new(&include_bytes!("../assets/sound/click.ogg")[..]);

        Self {
            backgrounds: BackgroundSounds {
                // Credits: Exit Exit by P C III (CC-BY)
                // https://freemusicarchive.org/music/P_C_III
                // https://soundcloud.com/pipe-choir-2/exit-exit
                exit_exit: Sound::new(
                    &include_bytes!("../assets/music/P C III - Exit Exit.ogg")[..],
                ),
                //https://freemusicarchive.org/music/P_C_III/earth2earth/earth2earth_1392
                // Credits: earth2earth by P C III (CC-BY)
                // https://freemusicarchive.org/music/P_C_III
                family_breaks: Sound::new(
                    &include_bytes!("../assets/music/P C III - The Family Breaks.ogg")[..],
                ),
                // https://freemusicarchive.org/music/P_C_III/The_Family_Breaks/The_Family_Breaks_1795
                // Credit: The Family Breaks by P C III (CC-BY)
                // https://freemusicarchive.org/music/P_C_III
                earth2earth: Sound::new(
                    &include_bytes!("../assets/music/P C III - earth2earth.ogg")[..],
                ),
            },
            effects: EffectSounds {
                walk: [walk_1, walk_2, walk_3, walk_4],
                monster_hit,
                monster_moved,
                explosion,
                player_hit,
                game_over,
                click,
            },
            sound_effect_muted: false,
            background_sound_queue,
            output_stream,
            rng,
        }
    }

    pub fn enqueue_background_music(&mut self, sound_data: Sound, delay: Duration) {
        if let Ok(source) = Decoder::new(sound_data) {
            self.background_sound_queue.append(source.delay(delay))
        }
    }

    pub fn play_sound(&mut self, effect: Effect, delay: Duration) {
        if self.sound_effect_muted {
            return;
        }
        let rng = &mut self.rng.clone();
        let mixer = self.output_stream.as_ref().map(OutputStream::mixer);
        let decoder = Decoder::new(self.data_from_effect(effect, rng));
        if let (Ok(sound), Some(mixer)) = (decoder, mixer) {
            let sink = Sink::connect_new(mixer);
            sink.append(sound.delay(delay));
            sink.detach();
        }
    }

    pub fn random_delay(&mut self) -> Duration {
        Duration::from_millis(self.rng.range_inclusive(1, 50).try_into().unwrap_or(0))
    }

    fn data_from_effect(&self, effect: Effect, rng: &mut Random) -> Sound {
        use Effect::*;
        match effect {
            Walk => rng
                .choose_with_fallback(&self.effects.walk, &self.effects.walk[0])
                .clone(),
            MonsterHit => self.effects.monster_hit.clone(),
            MonsterMoved => self.effects.monster_moved.clone(),
            Explosion => self.effects.explosion.clone(),
            PlayerHit => self.effects.player_hit.clone(),
            GameOver => self.effects.game_over.clone(),
            Click => self.effects.click.clone(),
        }
    }

    pub fn set_background_volume(&mut self, volume: f32) {
        let volume = volume.clamp(0.0, 1.0);

        // NOTE: we can't just pause the playback here based on the
        // volume, because `pause`/`play` is directly controlled by
        // the end-user code.
        //
        // If we do want to do that here (and it would be cleaner I
        // think, rather than just setting volume to 0), we have to
        // make `self.background_sound_queue` private and provide an
        // API for the things the game code does now directly.
        //
        // That's possibly worthwhile, but a future enhancement.
        self.background_sound_queue.set_volume(volume);
    }

    pub fn set_effects_volume(&mut self, volume: f32) {
        self.sound_effect_muted = volume == 0.0;
    }
}

impl Default for Audio {
    fn default() -> Self {
        Audio::new()
    }
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
    pub player_hit: Sound,
    pub game_over: Sound,
    pub click: Sound,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Effect {
    Walk,
    MonsterHit,
    MonsterMoved,
    Explosion,
    PlayerHit,
    GameOver,
    Click,
}
