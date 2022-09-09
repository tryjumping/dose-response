use std::time::{Duration, Instant};

use serde::{Deserialize, Serialize};

#[derive(Copy, Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Timer {
    max: Duration,
    current: Duration,
    current_frames: i32,
    max_frames: i32,
}

impl Timer {
    pub fn new(duration: Duration) -> Self {
        // NOTE: this will mess up animation timing on non-60 FPS
        // TODO: move this to formula/constants
        // TODO: actually enforce the 60 FPS
        let fps = 60.0;
        let dt_ms = 1000.0 / fps;
        let duration_ms = duration.as_secs_f32() * 1000.0;
        let duration_frame_count = (duration_ms / dt_ms).floor() as i32;
        Timer {
            max: duration,
            current: duration,
            current_frames: duration_frame_count,
            max_frames: duration_frame_count,
        }
    }

    pub fn new_elapsed(duration: Duration, elapsed_percent: f32) -> Self {
        assert!(elapsed_percent >= 0.0);
        assert!(elapsed_percent <= 1.0);
        let mut timer = Timer::new(duration);
        let current_ms = duration.as_secs_f32() * 1000.0 * (1.0 - elapsed_percent);
        assert!(current_ms >= 0.0);
        timer.current = Duration::from_millis(current_ms as u64);
        timer.current_frames = ((timer.max_frames as f32) * (1.0 - elapsed_percent)).floor() as i32;
        timer
    }

    pub fn update(&mut self, dt: Duration) {
        if self.current_frames > 0 {
            self.current_frames -= 1;
        }
        if dt > self.current {
            self.current = Duration::new(0, 0);
        } else {
            self.current -= dt;
        }
    }

    pub fn finish(&mut self) {
        self.current_frames = 0;
        self.current = Duration::new(0, 0);
    }

    pub fn reset(&mut self) {
        self.current = self.max;
        self.current_frames = self.max_frames;
    }

    pub fn percentage_remaining(&self) -> f32 {
        self.current.as_secs_f32() / self.max.as_secs_f32()
    }

    pub fn percentage_elapsed(&self) -> f32 {
        1.0 - self.percentage_remaining()
    }

    pub fn finished(&self) -> bool {
        // NOTE: basing whether the timer is finished on the Duration
        // seems to result in a smoother experience. At least in the
        // debug mode. Probably due to the fact that the variable dt
        // smooths any framerate variations out.
        //
        // We need the deterministic current frames thing, but if this
        // doesn't feel well, we'll need to do something. Like maybe
        // switch this based on a feature flag.
        //
        //self.current == Duration::new(0, 0);
        self.current_frames == 0
    }
}

#[derive(Copy, Clone)]
pub struct Stopwatch {
    start: Instant,
}

impl Stopwatch {
    pub fn start() -> Self {
        Stopwatch {
            start: Instant::now(),
        }
    }

    pub fn finish(self) -> Duration {
        Instant::now().duration_since(self.start)
    }
}
