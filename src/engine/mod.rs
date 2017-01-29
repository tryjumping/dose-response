use std::borrow::Cow;

use color::Color;
use point::Point;


pub mod tcod;


#[derive(Debug, Clone)]
pub enum Draw {
    Char(Point, char, Color),
    Text(Point, Cow<'static, str>, Color),
    Background(Point, Color),
    Rectangle(Point, Point, Color),
    Fade(f32, Color),
}


/// Settings the engine needs to carry.
///
/// Things such as the fullscreen/windowed display, font size, font
/// type, etc.
#[derive(Debug, Clone, Copy)]
pub struct Settings {
    pub fullscreen: bool,
}
