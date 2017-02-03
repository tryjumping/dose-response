use time::Duration;

use color::Color;
use engine::Draw;
use point::Point;


pub trait Render {
    fn render(&self, dt: Duration) -> (char, Color, Option<Color>);
}

pub fn draw<R: Render>(drawcalls: &mut Vec<Draw>, dt: Duration, pos: Point, render: &R) {
    use engine::Draw::*;
    let (glyph, fg, bg_opt) = render.render(dt);
    if let Some(background) = bg_opt {
        drawcalls.push(Background(pos, background));
    }
    drawcalls.push(Char(pos, glyph, fg));
}


pub fn progress_bar(drawcalls: &mut Vec<Draw>, percentage: f32, pos: Point, width: i32,
                    foreground: Color, background: Color) {
        assert!(percentage >= 0.0);
        assert!(percentage <= 1.0);
        for x_increment in 0..width {
            let x_percent = x_increment as f32 / width as f32;
            // TODO: max is not a full bar
            let color = if percentage == 1.0 {
                foreground
            } else if percentage == 0.0 {
                background
            } else if   x_percent >= percentage {
                background
            } else {
                foreground
            };
            drawcalls.push(Draw::Background(pos + (x_increment, 0), color));
        }
}


pub fn lerp(from: f32, to: f32, t: f32) -> f32 {
    (1.0 - t) * from + t * to
}

pub fn fade_color(from: Color, to: Color, progress: f32) -> Color {
    debug_assert!(progress >= 0.0);
    debug_assert!(progress <= 1.0);
    Color {
        r: lerp(from.r as f32, to.r as f32, progress) as u8,
        g: lerp(from.g as f32, to.g as f32, progress) as u8,
        b: lerp(from.b as f32, to.b as f32, progress) as u8,
    }
}
