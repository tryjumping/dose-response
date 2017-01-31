use std::borrow::Cow;

use time::Duration;

use color::Color;
use keys::Key;
use point::Point;


pub mod glium;
pub mod piston;
pub mod rustbox;
pub mod tcod;


#[derive(Debug, Clone)]
pub enum Draw {
    Char(Point, char, Color),
    Background(Point, Color),
    Text(Point, Cow<'static, str>, Color),
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


pub type UpdateFn<T> = fn(T,
                       dt: Duration,
                       size: Point,
                       fps: i32,
                       keys: &[Key],
                       settings: Settings,
                       drawcalls: &mut Vec<Draw>)
                       -> Option<(Settings, T)>;
