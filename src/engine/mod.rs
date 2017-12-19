use color::Color;
use game::RunningState;
use keys::Key;
use point::Point;
use state::State;
use std::borrow::Cow;

use std::time::Duration;


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


#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Mouse {
    pub tile_pos: Point,
    pub screen_pos: Point,
    pub left: bool,
    pub right: bool,
}

impl Default for Mouse {
    fn default() -> Self {
        Mouse {
            tile_pos: Point::new(-1, -1),
            screen_pos: Point::new(-1, -1),
            left: false,
            right: false,
        }
    }
}


/// Settings the engine needs to carry.
///
/// Things such as the fullscreen/windowed display, font size, font
/// type, etc.
#[derive(Debug, Clone, Copy)]
pub struct Settings {
    pub fullscreen: bool,
}


pub type UpdateFn = fn(&mut State,
                       dt: Duration,
                       size: Point,
                       fps: i32,
                       keys: &[Key],
                       mouse: Mouse,
                       settings: &mut Settings,
                       drawcalls: &mut Vec<Draw>)
                       -> RunningState;


// NOTE:
// fn texture_coords_from_char(chr: char) -> Option<(i32, i32)>
include!(concat!(env!("OUT_DIR"), "/glyph_lookup_table.rs"));
