use std::time::Duration;
#[cfg(not(feature = "web"))]
use std::time::Instant;

use util;

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
        let current_ms = util::num_milliseconds(duration) as f32 * (1.0 - elapsed_percent);
        assert!(current_ms >= 0.0);
        timer.current = Duration::from_millis(current_ms as u64);

        timer
    }

    pub fn update(&mut self, dt: Duration) {
        if dt > self.current {
            self.current = Duration::new(0, 0);
        } else {
            self.current = self.current - dt;
        }
    }

    pub fn percentage_remaining(&self) -> f32 {
        (util::num_milliseconds(self.current) as f32) / (util::num_milliseconds(self.max) as f32)
    }

    pub fn percentage_elapsed(&self) -> f32 {
        1.0 - self.percentage_remaining()
    }

    pub fn finished(&self) -> bool {
        self.current == Duration::new(0, 0)
    }
}

pub struct Stopwatch {
    #[cfg(not(feature = "web"))] start: Instant,
}

impl Stopwatch {
    pub fn start() -> Self {
        Stopwatch {
            #[cfg(not(feature = "web"))]
            start: Instant::now(),
        }
    }

    pub fn finish(self) -> Duration {
        #[cfg(not(feature = "web"))]
        return Instant::now().duration_since(self.start);

        // TODO: make this work for the web as well!
        #[cfg(feature = "web")]
        return Duration::new(0, 0);
    }
}
