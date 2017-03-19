use std::borrow::Cow;

use time::Duration;

use color::Color;
use keys::Key;
use point::Point;


#[cfg(feature = "opengl")]
pub mod glium;

#[cfg(feature = "piston")]
pub mod piston;

#[cfg(feature = "libtcod")]
pub mod tcod;

#[cfg(feature = "terminal")]
pub mod rustbox;

#[cfg(feature = "remote")]
pub mod remote;


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


// NOTE:
// fn texture_coords_from_char(chr: char) -> Option<(i32, i32)>
include!(concat!(env!("OUT_DIR"), "/glyph_lookup_table.rs"));
