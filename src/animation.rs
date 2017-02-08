use color::Color;
use point::{Point, SquareArea};
use timer::Timer;

use time::Duration;

pub trait AreaOfEffect {
    fn update(&mut self, dt: Duration);
    fn finished(&self) -> bool;
    fn covered_tiles(&self) -> Box<Iterator<Item=Point>>;
    fn render(&self) -> Box<Iterator<Item=(Point, Color)>>;
}

#[derive(Debug)]
pub struct SquareExplosion {
    pub center: Point,
    pub max_radius: i32,
    pub initial_radius: i32,
    pub current_radius: i32,
    pub color: Color,
    pub wave_count: i32,
    pub timer: Timer,
}

impl SquareExplosion {
    pub fn new(center: Point, max_radius: i32, initial_radius: i32, color: Color) -> Self {
        assert!(initial_radius <= max_radius);
        // Count the initial wave plus the rest that makes the difference
        let wave_count = max_radius - initial_radius + 1;
        let wave_duration = Duration::milliseconds(100);
        SquareExplosion {
            center: center,
            max_radius: max_radius,
            initial_radius: initial_radius,
            current_radius: initial_radius,
            color: color,
            wave_count: wave_count,
            timer: Timer::new(wave_duration * wave_count),
        }
    }

}

impl AreaOfEffect for SquareExplosion {
    fn update(&mut self, dt: Duration) {
        if self.timer.finished() {
            // do nothing
        } else {
            self.timer.update(dt);
            let single_wave_percentage = 1.0 / (self.wave_count as f32);
            self.current_radius = self.initial_radius + (self.timer.percentage_elapsed() / single_wave_percentage) as i32;
            if self.current_radius > self.max_radius {
                self.current_radius = self.max_radius;
            }
        }
    }

    fn finished(&self) -> bool {
        self.timer.finished()
    }

    fn covered_tiles(&self) -> Box<Iterator<Item=Point>> {
        Box::new(SquareArea::new(self.center, self.max_radius))
    }

    fn render(&self) -> Box<Iterator<Item=(Point, Color)>> {
        let color = self.color;
        Box::new(
            SquareArea::new(self.center, self.current_radius)
                .map(move |pos| (pos, color)))
    }

}



#[derive(Debug)]
pub struct ScreenFade {
    pub color: Color,
    pub fade_out_time: Duration,
    pub wait_time: Duration,
    pub fade_in_time: Duration,
    pub timer: Timer,
    pub phase: ScreenFadePhase,
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum ScreenFadePhase {
    FadeOut,
    Wait,
    FadeIn,
    Done,
}

impl ScreenFade {
    pub fn new(color: Color, fade_out: Duration, wait: Duration, fade_in: Duration) -> Self {
        ScreenFade {
            color: color,
            fade_out_time: fade_out,
            wait_time: wait,
            fade_in_time: fade_in,
            timer: Timer::new(fade_out),
            phase: ScreenFadePhase::FadeOut,
        }
    }

    pub fn update(&mut self, dt: Duration) {
        self.timer.update(dt);
        if self.timer.finished() {
            match self.phase {
                ScreenFadePhase::FadeOut => {
                    self.timer = Timer::new(self.wait_time);
                    self.phase = ScreenFadePhase::Wait;
                }
                ScreenFadePhase::Wait => {
                    self.timer = Timer::new(self.fade_in_time);
                    self.phase = ScreenFadePhase::FadeIn;
                }
                ScreenFadePhase::FadeIn => {
                    self.phase = ScreenFadePhase::Done;
                }
                ScreenFadePhase::Done => {
                    // NOTE: we're done. Nothing to do here.
                }
            }
        }
    }
}
