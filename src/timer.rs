use time::Duration;


#[derive(Copy, Clone, Debug)]
pub struct Timer {
    max: Duration,
    current: Duration,
}

impl Timer {
    pub fn new(duration: Duration) -> Timer {
        Timer {
            max: duration,
            current: duration,
        }
    }

    pub fn update(&mut self, dt: Duration) {
        if dt > self.current {
            self.current = Duration::zero();
        } else {
            self.current = self.current - dt;
        }
    }

    pub fn percentage_remaining(&self) -> f32 {
        (self.current.num_milliseconds() as f32) / (self.max.num_milliseconds() as f32)
    }

    pub fn percentage_elapsed(&self) -> f32 {
        1.0 - self.percentage_remaining()
    }

    pub fn finished(&self) -> bool {
        self.current.is_zero()
    }

}
