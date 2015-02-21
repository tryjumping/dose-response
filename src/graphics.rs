use std::time::Duration;

use tcod::RootConsole;

use color::Color;
use engine::Display;


pub trait Render {
    fn render(&self, dt: Duration) -> (char, Color, Option<Color>);
}


#[derive(Copy, Debug)]
pub enum Animation {
    None,
    ForegroundCycle{from: Color, to: Color, duration: Duration},
}

pub fn draw<R: Render>(display: &mut Display, dt: Duration,
                       pos: (i32, i32), render: &R) {
    let (x, y) = pos;
    let (glyph, fg, bg_opt) = render.render(dt);
    let bg = match bg_opt {
        Some(col) => col,
        // TODO: don't set the background at all if it's not passed in:
        None => RootConsole.get_char_background(x, y)
    };
    display.draw_char(x, y, glyph, fg, bg);
}


pub fn fade_color(from: Color, to: Color, progress: f32) -> Color {
    from.lerp(to, progress)
}
