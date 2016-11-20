use time::Duration;

use color::Color;
use engine::Display;
use point::Point;


pub trait Render {
    fn render(&self, dt: Duration) -> (char, Color, Option<Color>);
}


#[derive(Copy, Clone, Debug)]
pub enum Animation {
    None,
    ForegroundCycle{from: Color, to: Color, duration: Duration},
}

pub fn draw<R: Render>(display: &mut Display, dt: Duration,
                       pos: Point, render: &R) {
    let (glyph, fg, bg_opt) = render.render(dt);
    let bg = match bg_opt {
        Some(col) => col,
        // TODO: don't set the background at all if it's not passed in:
        None => display.get_background(pos)
    };
    display.draw_char(pos, glyph, fg, bg);
}


pub fn fade_color(from: Color, to: Color, progress: f32) -> Color {
    ::tcod::colors::lerp(from, to, progress)
}
