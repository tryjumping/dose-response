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
        None => color::background,
    };
    display.draw_char(x, y, glyph, fg, bg);
}
