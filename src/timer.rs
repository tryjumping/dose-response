use std::time::{Duration, Instant};

use serde::{Deserialize, Serialize};

#[derive(Copy, Clone, Debug, Serialize, Deserialize)]
pub struct Timer {
    max: Duration,
    current: Duration,
}

impl Timer {
    pub fn new(duration: Duration) -> Self {
        Timer {
            max: duration,
            current: duration,
        }
    }

    pub fn new_elapsed(duration: Duration, elapsed_percent: f32) -> Self {
        assert!(elapsed_percent >= 0.0);
        assert!(elapsed_percent <= 1.0);
        let mut timer = Timer::new(duration);
        let current_ms = duration.as_secs_f32() * 1000.0 * (1.0 - elapsed_percent);
        assert!(current_ms >= 0.0);
        timer.current = Duration::from_millis(current_ms as u64);

        timer
    }

    pub fn update(&mut self, dt: Duration) {
        if dt > self.current {
            self.current = Duration::new(0, 0);
        } else {
            self.current -= dt;
        }
    }

    pub fn finish(&mut self) {
        self.current = Duration::new(0, 0);
    }

    pub fn reset(&mut self) {
        self.current = self.max;
    }

    pub fn percentage_remaining(&self) -> f32 {
        self.current.as_secs_f32() / self.max.as_secs_f32()
    }

    pub fn percentage_elapsed(&self) -> f32 {
        1.0 - self.percentage_remaining()
    }

    pub fn finished(&self) -> bool {
        self.current == Duration::new(0, 0)
    }
}

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
