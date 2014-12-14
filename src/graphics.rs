use std::time::Duration;

use color::{mod, Color};
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
    if progress <= 0f32 {
        return from;
    } else if progress >= 1f32 {
        return to;
    };
    let dr = (to.r as f32 - from.r as f32) * progress;
    let dg = (to.g as f32 - from.g as f32) * progress;
    let db = (to.b as f32 - from.b as f32) * progress;
    Color {
        r: (from.r as f32 + dr) as u8,
        g: (from.g as f32 + dg) as u8,
        b: (from.b as f32 + db) as u8,
    }
}
