use color::Color;
use engine::Display;


pub trait Render {
    fn render(&self) -> (char, Color, Color);
}



pub fn draw<R: Render>(display: &mut Display, pos: (int, int), render: &R) {
    let (x, y) = pos;
    let (glyph, fg, bg) = render.render();
    display.draw_char(x, y, glyph, fg, bg);
}
