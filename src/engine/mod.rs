use color::{Color, ColorAlpha};
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
    /// Position, text, colour
    Text(Point, Cow<'static, str>, Color, TextOptions),
    /// Rectangle, color
    Rectangle(Rectangle, Color),
    /// Fade (one minus alpha: 1.0 means no fade, 0.0 means full fade), color
    Fade(f32, Color),
}


/// The drawcalls that the engine will process and render.
enum Drawcall {
    Rectangle(Option<Rectangle>, ColorAlpha),
    Image(Rectangle, Rectangle, Color),
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


#[derive(Copy, Clone, Debug)]
pub struct Cell {
    pub glyph: char,
    pub foreground: Color,
    pub background: Color,
    pub offset_px: Point,
}

impl Default for Cell {
    fn default() -> Self {
        Cell {
            glyph: ' ',
            foreground: Color{ r: 0, g: 0, b: 0},
            background: Color{ r: 255, g: 0, b: 255},
            offset_px: Point::zero(),
        }
    }
}


pub struct BackgroundMap {
    display_size: Point,
    padding: Point,
    map: Vec<Cell>,
}

#[allow(dead_code)]
impl BackgroundMap {
    pub fn new(display_size: Point, padding: Point) -> Self {
        assert!(display_size > Point::zero());
        assert!(padding >= Point::zero());
        let size = display_size + (padding * 2);
        BackgroundMap {
            display_size,
            padding,
            map: vec![Default::default(); (size.x * size.y) as usize],
        }
    }

    pub fn clear(&mut self, background: Color) {
        for cell in self.map.iter_mut() {
            *cell = Cell { background, ..Default::default() };
        }
    }

    pub fn size(&self) -> Point {
        self.display_size + (self.padding * 2)
    }

    fn index(&self, pos: Point) -> Option<usize> {
        if self.contains(pos) {
            let pos = pos + self.padding;
            Some((pos.y * self.size().x + pos.x) as usize)
        } else {
            None
        }
    }

    pub fn contains(&self, pos: Point) -> bool {
        let min = Point::zero() - self.padding;
        let max = self.display_size + self.padding;

        pos.x >= min.x && pos.y >= min.y && pos.x < max.x && pos.y < max.y
    }

    pub fn set(&mut self, pos: Point, glyph: char, foreground: Color, background: Color, offset_px: Point) {
        if let Some(ix) = self.index(pos) {
            self.map[ix] = Cell { glyph, foreground, background, offset_px };
        }
    }

    pub fn set_glyph(&mut self, pos: Point, glyph: char, foreground: Color, offset_px: Point) {
        if let Some(ix) = self.index(pos) {
            self.map[ix].glyph = glyph;
            self.map[ix].foreground = foreground;
            self.map[ix].offset_px = offset_px;
        }
    }

    pub fn set_background(&mut self, pos: Point, background: Color) {
        if let Some(ix) = self.index(pos) {
            self.map[ix].background = background;
        }
    }

    pub fn get(&self, pos: Point) -> Color {
        if let Some(ix) = self.index(pos) {
            self.map[ix].background
        } else {
            Default::default()
        }
    }

    pub fn cells(&self) -> impl Iterator<Item=(Point, &Cell)> {
        let padding = self.padding;
        let width = self.size().x;
        self.map
            .iter()
            .enumerate()
            .map(move |(index, cell)| {
                let pos = Point::new(index as i32 % width, index as i32 / width);
                let pos = pos - padding;
                (pos, cell)
            })
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
    map: &mut BackgroundMap,
    drawcalls: &mut Vec<Draw>,
) -> RunningState;

// NOTE:
// fn texture_coords_from_char(chr: char) -> Option<(i32, i32)>
#[cfg(not(feature = "web"))]
include!(concat!(env!("OUT_DIR"), "/glyph_lookup_table.rs"));
