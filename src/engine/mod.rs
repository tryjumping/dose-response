use color::Color;
use game::RunningState;
use keys::Key;
use point::Point;
use state::State;
use std::borrow::Cow;

use std::ops::RangeFrom;
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


// NOTE: This is designed specifically to deduplicated characters on
// the same position (using Vec::dedup). So the only thing considered
// equal are characters with the same pos value.
impl PartialEq for Draw {
    fn eq(&self, other: &Self) -> bool {
        use engine::Draw::*;
        match (self, other) {
            (&Char(p1, ..), &Char(p2, ..)) => p1 == p2,
            _ => false,
        }
    }
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


/// Sort the drawcalls in the specified range. Then "reverse
/// deduplicate" them -- meaning the latest item stays (rather than
/// the first one in a normal dedup).
pub fn sort_drawcalls(drawcalls: &mut Vec<Draw>, range: RangeFrom<usize>) {
    use std::cmp::Ordering::*;
    use engine::Draw::*;
    drawcalls[range].sort_by(|a, b| {
        match (a, b) {
            (&Char(p1, ..), &Char(p2, ..)) => {
                let x_ordering = p1.x.cmp(&p2.x);
                if x_ordering == Equal {
                    p1.y.cmp(&p2.y)
                } else {
                    x_ordering
                }
            }

            (&Background(..), &Background(..)) => Equal,
            (&Text(..), &Text(..)) => Equal,
            (&Rectangle(..), &Rectangle(..)) => Equal,
            (&Fade(..), &Fade(..)) => Equal,

            (&Fade(..), _) => Greater,
            (_, &Fade(..)) => Less,

            (&Background(..), &Char(..)) => Less,
            (&Char(..), &Background(..)) => Greater,

            (&Background(..), &Text(..)) => Less,
            (&Text(..), &Background(..)) => Greater,

            (&Background(..), &Rectangle(..)) => Less,
            (&Rectangle(..), &Background(..)) => Greater,

            _ => Equal,
        }
    });

    // Remove duplicate background and foreground tiles. I.e. for
    // any given point, only the last specified drawcall of the
    // same kind will remain.
    drawcalls.reverse();
    drawcalls.dedup();
    drawcalls.reverse();
}


// NOTE:
// fn texture_coords_from_char(chr: char) -> Option<(i32, i32)>
include!(concat!(env!("OUT_DIR"), "/glyph_lookup_table.rs"));
