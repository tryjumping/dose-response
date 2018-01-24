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

#[cfg(feature = "web")]
pub mod wasm;


#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Draw {
    /// Position, glyph, color
    Char(Point, char, Color),
    /// Position, color
    Background(Point, Color),
    /// Position, text, colour
    Text(Point, Cow<'static, str>, Color, TextOptions),
    /// Position, size, color
    Rectangle(Point, Point, Color),
    /// Fade (one minus alpha: 1.0 means no fade, 0.0 means full fade), color
    Fade(f32, Color),
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct TextOptions {
    /// Regular old text alignment: left, center, right.
    pub align: TextAlign,

    /// Whether to wrap the text.
    pub wrap: bool,

    /// If less than `1`, ignore it. Used for text wrapping and
    /// centering.
    pub width: i32,
}


impl TextOptions {
    pub fn align_right() -> TextOptions {
        TextOptions {
            align: TextAlign::Right,
            .. Default::default()
        }
    }


    pub fn align_center(width: i32) -> TextOptions {
        TextOptions {
            align: TextAlign::Center,
            width: width,
            .. Default::default()
        }
    }

}


impl Default for TextOptions {
    fn default() -> Self {
        TextOptions {
            align: TextAlign::Left,
            wrap: false,
            width: 0,
        }
    }
}



#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub enum TextAlign {
    Left,
    Right,
    Center,
}


pub trait TextMetrics {
    /// Return the height in tiles of the given text.
    ///
    /// Panics when `text_drawcall` is not `Draw::Text`
    fn get_text_height(&self, text_drawcall: &Draw) -> i32;
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


#[allow(dead_code)]
pub type UpdateFn = fn(&mut State,
                       dt: Duration,
                       size: Point,
                       fps: i32,
                       keys: &[Key],
                       mouse: Mouse,
                       settings: &mut Settings,
                       metrics: &TextMetrics,
                       drawcalls: &mut Vec<Draw>)
                       -> RunningState;




// NOTE:
// fn texture_coords_from_char(chr: char) -> Option<(i32, i32)>
include!(concat!(env!("OUT_DIR"), "/glyph_lookup_table.rs"));
