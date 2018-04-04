use color::Color;
use game::RunningState;
use keys::Key;
use point::Point;
use rect::Rectangle;
use state::State;
use std::borrow::Cow;

use std::time::Duration;

#[cfg(feature = "opengl")]
pub mod glium;

#[cfg(feature = "sdl")]
pub mod sdl;

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

pub const DRAWCALL_CAPACITY: usize = 8000;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Draw {
    /// Tile Position, glyph, color, pixel offset
    Char(Point, char, Color, Point),
    /// Position, color
    Background(Point, Color),
    /// Position, text, colour
    Text(Point, Cow<'static, str>, Color, TextOptions),
    /// Rectangle, color
    Rectangle(Rectangle, Color),
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
    pub fn align_left() -> TextOptions {
        TextOptions {
            align: TextAlign::Left,
            ..Default::default()
        }
    }
    pub fn align_right() -> TextOptions {
        TextOptions {
            align: TextAlign::Right,
            ..Default::default()
        }
    }

    pub fn align_center(width: i32) -> TextOptions {
        TextOptions {
            align: TextAlign::Center,
            width: width,
            ..Default::default()
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

    /// Return the width in tiles of the given text.
    ///
    /// Panics when `text_drawcall` is not `Draw::Text`
    fn get_text_width(&self, text_drawcall: &Draw) -> i32;

    /// Return the width and height of the given text in tiles.
    ///
    /// Panics when `text_drawcall` is not `Draw::Text`
    fn text_size(&self, text_drawcall: &Draw) -> Point {
        Point::new(
            self.get_text_width(text_drawcall),
            self.get_text_height(text_drawcall),
        )
    }

    /// Return the rectangle the text will be rendered in.
    ///
    /// Panics when `text_drawcall` is not `Draw::Text`
    fn text_rect(&self, text_drawcall: &Draw) -> Rectangle {
        match text_drawcall {
            &Draw::Text(start_pos, _, _, options) => {
                let size = self.text_size(text_drawcall);

                let top_left = if options.wrap && options.width > 0 {
                    start_pos
                } else {
                    use engine::TextAlign::*;
                    match options.align {
                        Left => start_pos,
                        Right => start_pos + (1 - size.x, 0),
                        Center => {
                            if options.width < 1 || (size.x > options.width) {
                                start_pos
                            } else {
                                start_pos + Point::new((options.width - size.x) / 2, 0)
                            }
                        }
                    }
                };

                Rectangle::from_point_and_size(top_left, size)
            }

            _ => {
                panic!("The argument to `TextMetrics::text_rect` must be `Draw::Text`!");
            }
        }
    }
}


// Calculate the width in pixels of a given text
fn text_width_px(text: &str, tile_width_px: i32) -> i32 {
    text.chars()
        .map(|chr| glyph_advance_width(chr).unwrap_or(tile_width_px as i32))
        .sum()
}

fn wrap_text(text: &str, width_tiles: i32, tile_width_px: i32) -> Vec<String> {
    let mut result = vec![];
    let wrap_width_px = width_tiles * tile_width_px;
    let space_width = glyph_advance_width(' ').unwrap_or(tile_width_px as i32);

    let mut current_line = String::new();
    let mut current_width_px = 0;

    let mut words = text.split(' ');
    if let Some(word) = words.next() {
        current_width_px += text_width_px(word, tile_width_px);
        current_line.push_str(word);
    }

    for word in words {
        let word_width = text_width_px(word, tile_width_px);
        if current_width_px + space_width + word_width <= wrap_width_px {
            current_width_px += space_width + word_width;
            current_line.push(' ');
            current_line.push_str(word);
        } else {
            result.push(current_line);
            current_width_px = word_width;
            current_line = String::from(word);
        }
    }
    result.push(current_line);

    result
}


#[derive(Clone, Copy, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct Mouse {
    pub tile_pos: Point,
    pub screen_pos: Point,
    pub left: bool,
    pub right: bool,
}

impl Mouse {
    pub fn new() -> Self {
        Default::default()
    }
}

pub fn populate_background_map(
    background_map: &mut Vec<Color>,
    display_size: Point,
    drawcalls: &Vec<Draw>,
) {
    assert!(background_map.len() >= (display_size.x * display_size.y) as usize);

    // NOTE: Clear the background_map by setting it to the default colour
    for color in background_map.iter_mut() {
        *color = Color { r: 0, g: 0, b: 0 };
    }

    // NOTE: generate the background map
    for drawcall in drawcalls {
        match drawcall {
            &Draw::Background(pos, background_color) => {
                if pos.x >= 0 && pos.y >= 0 && pos.x < display_size.x && pos.y < display_size.y {
                    background_map[(pos.y * display_size.x + pos.x) as usize] = background_color;
                }
            }

            &Draw::Rectangle(rect, color) => for pos in rect.points() {
                if pos.x >= 0 && pos.y >= 0 && pos.x < display_size.x && pos.y < display_size.y {
                    background_map[(pos.y * display_size.x + pos.x) as usize] = color;
                }
            },

            _ => {}
        }
    }
}

/// Settings the engine needs to carry.
///
/// Things such as the fullscreen/windowed display, font size, font
/// type, etc.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Settings {
    pub fullscreen: bool,
}

#[allow(dead_code)]
pub type UpdateFn = fn(
    &mut State,
    dt: Duration,
    size: Point,
    fps: i32,
    keys: &[Key],
    mouse: Mouse,
    settings: &mut Settings,
    metrics: &TextMetrics,
    drawcalls: &mut Vec<Draw>,
) -> RunningState;

// NOTE:
// fn texture_coords_from_char(chr: char) -> Option<(i32, i32)>
#[cfg(not(feature = "web"))]
include!(concat!(env!("OUT_DIR"), "/glyph_lookup_table.rs"));
