use time::Duration;

use color::Color;
use engine::{Display, Draw};
use point::Point;


pub trait Render {
    fn render(&self, dt: Duration) -> (char, Color, Option<Color>);
}

pub fn draw<R: Render>(display: &mut Display, drawcalls: &mut Vec<Draw>, dt: Duration,
                       pos: Point, render: &R) {
    use engine::Draw::*;
    let (glyph, fg, bg_opt) = render.render(dt);
    drawcalls.push(Char(pos, glyph, fg));
    if let Some(background) = bg_opt {
        drawcalls.push(Background(pos, background));
    }
}


pub fn fade_color(from: Color, to: Color, progress: f32) -> Color {
    ::tcod::colors::lerp(from, to, progress)
}
