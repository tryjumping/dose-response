use std::time::Duration;

use color::{mod, Color};
use tcod;
use engine::Display;


pub trait Render {
    fn render(&self, dt: Duration) -> (char, Color, Option<Color>);
}


#[deriving(Copy, Show)]
pub enum Animation {
    None,
    ForegroundCycle{from: Color, to: Color, duration: Duration},
}

pub fn draw<R: Render>(display: &mut Display, dt: Duration,
                       pos: (int, int), render: &R) {
    let (x, y) = pos;
    let (glyph, fg, bg_opt) = render.render(dt);
    let bg = match bg_opt {
        Some(col) => col,
        // TODO: don't set the background at all if it's not passed in:
        None => color::background,
    };
    display.draw_char(x, y, glyph, fg, bg);
}


pub fn fade_color(from: Color, to: Color, progress: f32) -> Color {
    // TODO: expose this from tcod-rs
    unsafe {
        tcod::ffi::TCOD_color_lerp(from, to, progress)
    }
}
