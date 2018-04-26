use color::{self, Color, ColorAlpha};
use game::RunningState;
use keys::Key;
use point::Point;
use rect::Rectangle;
use state::State;
use ui::Button;
use util;

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


/// The drawcalls that the engine will process and render.
#[derive(Debug, Copy, Clone)]
pub enum Drawcall {
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
    fn get_text_height(&self, text: &str, options: TextOptions) -> i32;

    /// Return the width in tiles of the given text.
    ///
    /// Panics when `text_drawcall` is not `Draw::Text`
    fn get_text_width(&self, text: &str, options: TextOptions) -> i32;

    /// Return the width and height of the given text in tiles.
    ///
    /// Panics when `text_drawcall` is not `Draw::Text`
    fn text_size(&self, text: &str, options: TextOptions) -> Point {
        Point::new(
            self.get_text_width(text, options),
            self.get_text_height(text, options),
        )
    }

    /// Return the rectangle the text will be rendered in.
    ///
    /// Panics when `text_drawcall` is not `Draw::Text`
    fn text_rect(&self, start_pos: Point, text: &str, options: TextOptions) -> Rectangle {
        let size = self.text_size(text, options);
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

    fn button_rect(&self, button: &Button) -> Rectangle {
        self.text_rect(button.pos, &button.text, button.text_options)
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


pub struct Display {
    display_size: Point,
    tilesize: i32,
    padding: Point,
    map: Vec<Cell>,
    drawcalls: Vec<Drawcall>,
    pub fade: ColorAlpha,
}

#[allow(dead_code)]
impl Display {
    pub fn new(display_size: Point, padding: Point, tilesize: i32) -> Self {
        assert!(display_size > Point::zero());
        assert!(padding >= Point::zero());
        let size = display_size + (padding * 2);
        Display {
            display_size,
            padding,
            tilesize,
            map: vec![Default::default(); (size.x * size.y) as usize],
            drawcalls: Vec::with_capacity(DRAWCALL_CAPACITY),
            fade: color::invisible,
        }
    }

    pub fn clear(&mut self, background: Color) {
        for cell in self.map.iter_mut() {
            *cell = Cell { background, ..Default::default() };
        }
        self.drawcalls.clear();
        self.fade = color::invisible;
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

    /// Set the value (RGBA) to fade the screen with.
    /// Unlike alpha, the `fade` argument is inverted: 1.0 means no fade, 0.0 means fully faded.
    pub fn set_fade(&mut self, color: Color, fade: f32) {
        let fade = util::clampf(0.0, fade, 1.0);
        let fade = (fade * 255.0) as u8;
        self.fade = color.alpha(255 - fade);
    }

    /// Draw a rectangle of the given colour.
    pub fn draw_rectangle(&mut self, rect: Rectangle, color: Color) {
        let top_left_px = rect.top_left() * self.tilesize;
        let dimensions_px = rect.size() * self.tilesize;

        let rect = Rectangle::from_point_and_size(top_left_px, dimensions_px);
        self.drawcalls.push(Drawcall::Rectangle(Some(rect), color.into()));
    }

    /// Draw a Button
    pub fn draw_button(&mut self, button: &Button) {
        self.draw_text(button.pos, &button.text, button.color, button.text_options);
    }

    pub fn draw_text(&mut self, start_pos: Point, text: &str,
                     color: Color, options: TextOptions)
    {
        let tilesize = self.tilesize;
        let mut render_line = |pos_px: Point, line: &str| {
            let mut offset_x = 0;

            // TODO: we need to split this by words or it
            // won't do word breaks, split at punctuation,
            // etc.

            // TODO: also, we're no longer calculating the
            // line height correctly. Needs to be set on the
            // actual result here.
            for chr in line.chars() {
                let (texture_index_x, texture_index_y) = texture_coords_from_char(chr)
                    .unwrap_or((0, 0));

                let src = Rectangle::from_point_and_size(
                    Point::new(texture_index_x, texture_index_y) * self.tilesize,
                    Point::from_i32(self.tilesize));
                let dst = Rectangle::from_point_and_size(
                    pos_px + (offset_x, 0),
                    Point::from_i32(self.tilesize));

                self.drawcalls.push(Drawcall::Image(src, dst, color));

                let advance_width =
                    glyph_advance_width(chr).unwrap_or(self.tilesize);
                offset_x += advance_width;
            }
        };

        if options.wrap && options.width > 0 {
            // TODO: handle text alignment for wrapped text
            let lines = wrap_text(text, options.width, tilesize);
            for (index, line) in lines.iter().enumerate() {
                let pos = (start_pos + Point::new(0, index as i32)) * tilesize;
                render_line(pos, line);
            }
        } else {
            use engine::TextAlign::*;
            let pos = match options.align {
                Left => start_pos * tilesize,
                Right => {
                    (start_pos + (1, 0)) * tilesize
                        - Point::new(text_width_px(text, tilesize), 0)
                }
                Center => {
                    let text_width = text_width_px(text, tilesize);
                    let max_width = options.width * tilesize;
                    if max_width < 1 || (text_width > max_width) {
                        start_pos
                    } else {
                        (start_pos * tilesize) + Point::new((max_width - text_width) / 2, 0)
                    }
                }
            };
            render_line(pos, text);
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

    pub fn push_drawcalls(&self,
                          display_size_px: Point,
                          tilesize: i32,
                          drawcalls: &mut Vec<Drawcall>) {
        assert!(tilesize > 0);

        // Render the background tiles separately and before all the other drawcalls.
        for (pos, cell) in self.cells() {
            let (texture_index_x, texture_index_y) = texture_coords_from_char(cell.glyph)
                .unwrap_or((0, 0));
            let texture_src = Rectangle::from_point_and_size(
                Point::new(texture_index_x, texture_index_y) * tilesize,
                Point::from_i32(tilesize));
            let background_dst = Rectangle::from_point_and_size(
                Point::new(pos.x * tilesize + cell.offset_px.x,
                           pos.y * tilesize + cell.offset_px.y),
                Point::from_i32(tilesize));

            // NOTE: Center the glyphs in their cells
            let glyph_width = glyph_advance_width(cell.glyph).unwrap_or(tilesize);
            let x_offset = (tilesize as i32 - glyph_width) / 2;
            let glyph_dst = background_dst.offset(Point::new(x_offset, 0));

            if rect_intersects_area(background_dst, display_size_px) {
                drawcalls.push(Drawcall::Rectangle(Some(background_dst), cell.background.into()));
            }
            if rect_intersects_area(glyph_dst, display_size_px) {
                drawcalls.push(Drawcall::Image(texture_src, glyph_dst, cell.foreground));
            }
        }

        drawcalls.extend(self.drawcalls.iter());

        if self.fade.alpha > 0 {
            drawcalls.push(Drawcall::Rectangle(None, self.fade));
        }
    }

}


/// Returns `true` if the `Rectangle` intersects the area that starts at `(0, 0)`
fn rect_intersects_area(rect: Rectangle, area: Point) -> bool {
    rect.right() >= 0 &&
        rect.left() < area.x &&
        rect.top() < area.y &&
        rect.bottom() >= 0
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
    display: &mut Display,
) -> RunningState;

// NOTE:
// fn texture_coords_from_char(chr: char) -> Option<(i32, i32)>
#[cfg(not(feature = "web"))]
include!(concat!(env!("OUT_DIR"), "/glyph_lookup_table.rs"));
