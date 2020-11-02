#![allow(dead_code)]

use crate::{
    color::{self, Color, ColorAlpha},
    graphic::{self, Graphic, TILE_SIZE},
    point::Point,
    rect::Rectangle,
    ui::Button,
    util,
};

use serde::{Deserialize, Serialize};

#[cfg(feature = "glutin-backend")]
mod loop_state;

#[cfg(feature = "glutin-backend")]
pub mod glutin;

#[cfg(feature = "glutin-backend")]
pub mod opengl;

#[cfg(feature = "remote")]
pub mod remote;

pub const DRAWCALL_CAPACITY: usize = 8000;
pub const VERTEX_CAPACITY: usize = 50_000;
pub const VERTEX_COMPONENT_COUNT: usize = 9;
const VERTEX_BUFFER_CAPACITY: usize = VERTEX_COMPONENT_COUNT * VERTEX_CAPACITY;

#[derive(Copy, Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum Texture {
    Text,
    Glyph,
    Tilemap,
    Egui,
}

impl Into<f32> for Texture {
    fn into(self) -> f32 {
        match self {
            Texture::Text => 0.0,
            Texture::Glyph => 1.0,
            Texture::Tilemap => 2.0,
            Texture::Egui => 3.0,
        }
    }
}

/// The drawcalls that the engine will process and render.
#[derive(Copy, Clone, Debug, Serialize, Deserialize)]
pub enum Drawcall {
    Rectangle(Rectangle, ColorAlpha),
    Image(Texture, Rectangle, Rectangle, Color),
}

#[derive(Copy, Clone, Debug)]
#[repr(C)]
pub struct Vertex {
    /// ID of the texture to read from.
    ///
    /// `0.0` means the font
    /// `1.0` means the tilemap
    pub texture_id: f32,

    /// Position in the pixels.
    ///
    /// Note that this doesn't have to be an integer, so you can
    /// implement smooth positioning by using a fractional value.
    pub pos_px: [f32; 2],

    /// Tile position in the tile map / sprite sheet.
    ///
    /// The units can be either normalised u/v coordinates (ranging
    /// from 0 to 1) or pixel texture coords.
    ///
    /// Egui produces normalised coords, everything else uses pixel
    /// ones.
    ///
    /// If both values are negative, no texture will be rendered.
    /// Instead, the colour will be used to fill the background.
    /// That's how render rectangles.
    pub tile_pos: [f32; 2],

    /// Colour of the glyph. The glyphs are greyscale, so this is how
    /// we set the final colour.
    pub color: [f32; 4],
}

impl Vertex {
    #[allow(dead_code)]
    fn to_f32_array(&self) -> [f32; 9] {
        [
            self.texture_id.into(),
            self.pos_px[0],
            self.pos_px[1],
            self.tile_pos[0],
            self.tile_pos[1],
            self.color[0],
            self.color[1],
            self.color[2],
            self.color[3],
        ]
    }
}

impl Into<[f32; 4]> for ColorAlpha {
    fn into(self) -> [f32; 4] {
        [
            f32::from(self.rgb.r) / 255.0,
            f32::from(self.rgb.g) / 255.0,
            f32::from(self.rgb.b) / 255.0,
            f32::from(self.alpha) / 255.0,
        ]
    }
}

trait VertexStore {
    fn push(&mut self, _: Vertex);
    fn count(&self) -> usize;
}

impl VertexStore for Vec<Vertex> {
    fn push(&mut self, vertex: Vertex) {
        self.push(vertex)
    }

    fn count(&self) -> usize {
        self.len()
    }
}

impl VertexStore for Vec<f32> {
    fn push(&mut self, vertex: Vertex) {
        self.extend(&vertex.to_f32_array())
    }

    fn count(&self) -> usize {
        assert_eq!(self.len() % 9, 0);
        self.len() / 9
    }
}

impl VertexStore for Vec<u8> {
    fn push(&mut self, vertex: Vertex) {
        for value in &vertex.to_f32_array() {
            // NOTE: WASM specifies the little endian ordering
            let bits: u32 = value.to_bits().to_le();
            let b1: u8 = (bits & 0xff) as u8;
            let b2: u8 = ((bits >> 8) & 0xff) as u8;
            let b3: u8 = ((bits >> 16) & 0xff) as u8;
            let b4: u8 = ((bits >> 24) & 0xff) as u8;

            self.push(b1);
            self.push(b2);
            self.push(b3);
            self.push(b4);
        }
    }

    fn count(&self) -> usize {
        todo!()
    }
}

fn build_vertices<T: VertexStore>(
    drawcalls: &[Drawcall],
    vertices: &mut T,
    display_size: [f32; 2],
) {
    let (display_size_x, display_size_y) = (display_size[0], display_size[1]);
    for drawcall in drawcalls {
        match drawcall {
            // NOTE: Rectangle, ColorAlpha)
            Drawcall::Rectangle(rect, color) => {
                let top_left_px = rect.top_left();
                let size_px = rect.size();
                let (pos_x, pos_y) = (top_left_px.x as f32, top_left_px.y as f32);
                let (dim_x, dim_y) = (size_px.x as f32, size_px.y as f32);

                // NOTE: cut off the area that's not in the display.
                //
                // Any drawcalls that would be entirely invisible will
                // have been filtered by now. But we still need to
                // process drawcalls that are only partially visible.
                // So this fixes the rendered position and height to
                // do just that.
                let (pos_x, dim_x) = if pos_x < 0.0 {
                    (0.0, dim_x + pos_x)
                } else if pos_x + dim_x >= display_size_x {
                    (pos_x, display_size_x - pos_x)
                } else {
                    (pos_x, dim_x)
                };
                let (pos_y, dim_y) = if pos_y < 0.0 {
                    (0.0, dim_y + pos_y)
                } else if pos_y + dim_y >= display_size_y {
                    (pos_y, display_size_y - pos_y)
                } else {
                    (pos_y, dim_y)
                };

                // Tile position of [-1, -1] indicates an area colour fill rather than a texture blit
                let tile_pos = [-1.0, -1.0];
                // The value here doesn't matter, the shader ignores the texture for rectangles.
                let texture_id = 0.0;
                let color = (*color).into();

                vertices.push(Vertex {
                    texture_id,
                    pos_px: [pos_x, pos_y],
                    tile_pos,
                    color,
                });
                vertices.push(Vertex {
                    texture_id,
                    pos_px: [pos_x + dim_x, pos_y],
                    tile_pos,
                    color,
                });
                vertices.push(Vertex {
                    texture_id,
                    pos_px: [pos_x, pos_y + dim_y],
                    tile_pos,
                    color,
                });

                vertices.push(Vertex {
                    texture_id,
                    pos_px: [pos_x + dim_x, pos_y],
                    tile_pos,
                    color,
                });
                vertices.push(Vertex {
                    texture_id,
                    pos_px: [pos_x, pos_y + dim_y],
                    tile_pos,
                    color,
                });
                vertices.push(Vertex {
                    texture_id,
                    pos_px: [pos_x + dim_x, pos_y + dim_y],
                    tile_pos,
                    color,
                });
            }

            // NOTE: (Rectangle, Rectangle, Color)
            Drawcall::Image(texture, src, dst, color) => {
                let pixel_pos = dst.top_left();
                let (pos_x, pos_y) = (pixel_pos.x as f32, pixel_pos.y as f32);
                let (tile_width, tile_height) = (dst.width() as f32, dst.height() as f32);
                let (tilemap_x, tilemap_y) = (src.top_left().x as f32, src.top_left().y as f32);
                let (texture_width, texture_height) = match texture {
                    Texture::Text => (tile_width, tile_height),
                    Texture::Glyph => (tile_width, tile_height),
                    Texture::Tilemap => (TILE_SIZE as f32, TILE_SIZE as f32),
                    // NOTE: Egui shouldn't appear in drawcalls, adding it here for completeness
                    Texture::Egui => (tile_width, tile_height),
                };

                let texture_id = (*texture).into();
                let rgba: ColorAlpha = (*color).into();
                let color = rgba.into();

                // NOTE: draw the glyph
                vertices.push(Vertex {
                    texture_id,
                    pos_px: [pos_x, pos_y],
                    tile_pos: [tilemap_x, tilemap_y],
                    color,
                });
                vertices.push(Vertex {
                    texture_id,
                    pos_px: [pos_x + tile_width, pos_y],
                    tile_pos: [tilemap_x + texture_width, tilemap_y],
                    color,
                });
                vertices.push(Vertex {
                    texture_id,
                    pos_px: [pos_x, pos_y + tile_height],
                    tile_pos: [tilemap_x, tilemap_y + texture_height],
                    color,
                });

                vertices.push(Vertex {
                    texture_id,
                    pos_px: [pos_x + tile_width, pos_y],
                    tile_pos: [tilemap_x + texture_width, tilemap_y],
                    color,
                });
                vertices.push(Vertex {
                    texture_id,
                    pos_px: [pos_x, pos_y + tile_height],
                    tile_pos: [tilemap_x, tilemap_y + texture_height],
                    color,
                });
                vertices.push(Vertex {
                    texture_id,
                    pos_px: [pos_x + tile_width, pos_y + tile_height],
                    tile_pos: [tilemap_x + texture_width, tilemap_y + texture_height],
                    color,
                });
            }
        }
    }
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

    /// Height of the text in tiles.
    pub height: i32,

    /// The number of lines to skip rendering. Allows printing only a
    /// subset of the text e.g. for pagination.
    pub skip: i32,
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
            width,
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
            height: 1,
            skip: 0,
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
    fn tile_width_px(&self) -> i32;

    fn text_width_px(&self) -> i32;

    fn advance_width_px(&self, glyph: char) -> i32 {
        // NOTE: we're assuming that font_size and tilesize always match!
        let font_size = self.tile_width_px() as u32;
        glyph_advance_width(font_size, glyph).unwrap_or(self.tile_width_px())
    }

    /// Return the height in tiles of the given text.
    ///
    /// Panics when `text_drawcall` is not `Draw::Text`
    fn get_text_height(&self, text: &str, options: TextOptions) -> i32 {
        if options.wrap && options.width > 0 {
            let font_size = self.text_width_px() as u32;
            // TODO: this does a needless allocation by
            // returning Vec<String> we don't use here.
            let lines = wrap_text(&text, font_size, options.width, self.tile_width_px());
            lines.len() as i32
        } else {
            1
        }
    }

    /// Return the width in tiles of the given text.
    ///
    /// Panics when `text_drawcall` is not `Draw::Text`
    fn get_text_width(&self, text: &str, options: TextOptions) -> i32 {
        let font_size = self.text_width_px() as u32;
        let pixel_width = if options.wrap && options.width > 0 {
            // // TODO: handle text alignment for wrapped text
            let lines = wrap_text(text, font_size, options.width, self.tile_width_px());
            lines
                .iter()
                .map(|line| text_width_px(line, font_size, self.tile_width_px()))
                .max()
                .unwrap_or(0)
        } else {
            text_width_px(text, font_size, self.tile_width_px())
        };
        let tile_width = (pixel_width as f32 / self.tile_width_px() as f32).ceil();
        tile_width as i32
    }

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
            use crate::engine::TextAlign::*;
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
        let rect = self.text_rect(button.pos, &button.text, button.text_options);
        if rect.height() < button.text_options.height {
            let new_size = rect.size() + (0, button.text_options.height - rect.height());
            Rectangle::from_point_and_size(rect.top_left(), new_size)
        } else {
            rect
        }
    }
}

// Calculate the width in pixels of a given text
fn text_width_px(text: &str, font_size: u32, max_glyph_width_px: i32) -> i32 {
    text.chars()
        .map(|chr| glyph_advance_width(font_size, chr).unwrap_or(max_glyph_width_px as i32))
        .sum()
}

/// Wrap the string with a specified font size based on the width in tiles.
/// `tile_width_px` determines the width of a single tile.
/// TODO That kinda seems weird? Why do we need it instead of just setting pixels?
fn wrap_text(text: &str, font_size: u32, width_tiles: i32, tile_width_px: i32) -> Vec<String> {
    let mut result = vec![];
    let wrap_width_px = width_tiles * tile_width_px;
    let space_width = glyph_advance_width(font_size, ' ').unwrap_or(tile_width_px as i32);

    let mut current_line = String::new();
    let mut current_width_px = 0;

    let mut words = text.split(' ');
    if let Some(word) = words.next() {
        current_width_px += text_width_px(word, font_size, tile_width_px);
        current_line.push_str(word);
    }

    for word in words {
        let word_width = text_width_px(word, font_size, tile_width_px);
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

#[derive(Copy, Clone, Debug)]
pub struct DisplayInfo {
    /// Size of the entire rendering area in logical pixels. `display_px + extra_px`
    window_size_px: [f32; 2],

    /// Size of the actual area the Display struct can render to.
    ///
    /// This covers all the rendered tiles, but if the window size
    /// doesn't cover the tiles cleanly, this may be a bit smaller.
    display_px: [f32; 2],

    /// Size of the entire rendering area in physical pixels. `window_size_px * DPI`
    viewport_size: [f32; 2],

    /// Number of physical pixels per a logical one. E.g. `DPI` of
    /// `2.0` will render a single game pixel as four pixels (two in
    /// each dimension) on the display. Think "retina" displays on
    /// macs that have doubled the pixel density in order to enable
    /// greater text crispness and whatnot (which we're not using, but
    /// we need to handle DPI nonetheless).
    dpi: f32,
}

/// Calculate the dimensions to provide the largest display
/// area while maintaining the aspect ratio (and letterbox the
/// display).
fn calculate_display_info(
    window_size_px: [f32; 2],
    display_size_tiles: Point,
    tilesize_px: i32,
    dpi: f32,
) -> DisplayInfo {
    let tilecount_x = display_size_tiles.x as f32;
    let tilecount_y = display_size_tiles.y as f32;
    let tilesize = tilesize_px as f32;

    let unscaled_game_width = tilecount_x * tilesize;
    let unscaled_game_height = tilecount_y * tilesize;

    let display_px = [unscaled_game_width, unscaled_game_height];
    let viewport_size = [window_size_px[0] * dpi, window_size_px[1] * dpi];

    DisplayInfo {
        window_size_px,
        display_px,
        viewport_size,
        dpi,
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct Mouse {
    pub tile_pos: Point,
    pub screen_pos: Point,
    /// The left button has clicked. I.e. pressed and released.
    pub left_clicked: bool,
    /// The Right button was clicked, i.e. pressed and released.
    pub right_clicked: bool,
    /// The left button is being held down.
    pub left_is_down: bool,
    /// The right button is being held down.
    pub right_is_down: bool,

    /// The amount a mouse wheel has scrolled.
    pub scroll_delta: [f32; 2],
}

impl Mouse {
    pub fn new() -> Self {
        Default::default()
    }
}

#[derive(Copy, Clone, Debug)]
pub struct Cell {
    pub graphic: Graphic,
    pub foreground: Color,
    pub background: Color,
}

impl Default for Cell {
    fn default() -> Self {
        Cell {
            graphic: Graphic::Empty,
            foreground: Color { r: 0, g: 0, b: 0 },
            background: Color { r: 0, g: 0, b: 0 },
        }
    }
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum DrawResult {
    Fit,
    Overflow,
}

#[derive(Default)]
pub struct Display {
    // TODO: this is the actual game area size in tiles. Rename it to something that makes that clear
    display_size: Point,
    /// Size of the full window in pixels.
    pub screen_size_px: Point,
    pub tile_size: i32,
    pub text_size: i32,
    pub offset_px: Point,
    /// This is the padding that makes the game fit a display of a different ratio
    padding: Point,
    map: Vec<Cell>,
    drawcalls: Vec<Drawcall>,
    pub fade: ColorAlpha,
    clear_background_color: Option<Color>,
}

#[allow(dead_code)]
impl Display {
    /// Create a new Display.
    ///
    /// `screen_size_px`: dimension of the game windo in logical pixels.
    /// `tilesize`: size (in pixels) of a single tile. Tiles are square.
    pub fn new(screen_size_px: Point, tile_size: i32, text_size: i32) -> Self {
        assert!(screen_size_px > Point::zero());
        assert!(tile_size > 0);
        assert!(text_size > 0);
        // NOTE: this padding is only here to make the screen scroll smoothly.
        // Without it, the partial tiles would not appear.
        let display_size = {
            let size = screen_size_px / tile_size;
            let extra_x = if size.x * tile_size < screen_size_px.x {
                1
            } else {
                0
            };
            let extra_y = if size.y * tile_size < screen_size_px.y {
                1
            } else {
                0
            };
            size + Point::new(extra_x, extra_y)
        };
        let padding = Point::from_i32(1);
        let size = display_size + (padding * 2);
        log::info!("Creating the internal Display: {:?}", size);
        Display {
            display_size,
            screen_size_px,
            padding,
            tile_size,
            text_size,
            map: vec![Default::default(); (size.x * size.y) as usize],
            drawcalls: Vec::with_capacity(DRAWCALL_CAPACITY),
            fade: color::invisible,
            ..Default::default()
        }
    }

    pub fn clear(&mut self, background: Color) {
        for cell in &mut self.map {
            *cell = Cell {
                background,
                ..Default::default()
            };
        }
        self.drawcalls.clear();
        self.fade = color::invisible;
        self.clear_background_color = Some(background);
    }

    /// This is the full display size: game plus padding.
    pub fn full_size_with_padding_in_tiles(&self) -> Point {
        self.display_size + (self.padding * 2)
    }

    /// This is the size of the display as originally requested.
    ///
    /// There is no padding here -- this should correspond to what's
    /// shown on the screen when there is no scrolling or other
    /// shennanigans going on.
    pub fn size_without_padding(&self) -> Point {
        self.display_size
    }

    fn index(&self, pos: Point) -> Option<usize> {
        if self.contains(pos) {
            let pos = pos + self.padding;
            Some((pos.y * self.full_size_with_padding_in_tiles().x + pos.x) as usize)
        } else {
            None
        }
    }

    pub fn contains(&self, pos: Point) -> bool {
        let min = Point::zero() - self.padding;
        let max = self.display_size + self.padding;

        pos.x >= min.x && pos.y >= min.y && pos.x < max.x && pos.y < max.y
    }

    pub fn set(&mut self, pos: Point, graphic: Graphic, foreground: Color, background: Color) {
        if let Some(ix) = self.index(pos) {
            self.map[ix] = Cell {
                graphic,
                foreground,
                background,
            };
        }
    }

    pub fn set_graphic(&mut self, pos: Point, graphic: Graphic, foreground: Color) {
        if let Some(ix) = self.index(pos) {
            self.map[ix].graphic = graphic;
            self.map[ix].foreground = foreground;
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
        let top_left_px = rect.top_left() * self.tile_size;
        let dimensions_px = rect.size() * self.tile_size;

        let rect = Rectangle::from_point_and_size(top_left_px, dimensions_px);
        self.drawcalls.push(Drawcall::Rectangle(rect, color.into()));
    }

    /// Draw a Button
    pub fn draw_button(&mut self, button: &Button) {
        self.draw_text_in_tile_coordinates(
            button.pos,
            &button.text,
            button.color,
            button.text_options,
            self.tile_size,
        );
    }

    /// Draw a glyph at the given pixel position.
    ///
    /// No offset or tile structure is applied. This is a raw command
    /// for when you really need to position a character precisely.
    pub fn draw_glyph_abs_px(&mut self, x: i32, y: i32, glyph: char, color: Color) {
        let (texture_px_x, texture_px_y) =
            texture_coords_px_from_char(self.text_size as u32, glyph).unwrap_or((0, 0));

        let src = Rectangle::from_point_and_size(
            Point::new(texture_px_x, texture_px_y),
            Point::from_i32(self.text_size),
        );
        let dst = Rectangle::from_point_and_size(Point::new(x, y), Point::from_i32(self.text_size));

        self.drawcalls
            .push(Drawcall::Image(Texture::Text, src, dst, color));
    }

    /// Draw text with tile coordinates rather than pixels.
    ///
    /// Shorthand for `draw_text`
    pub fn draw_text_in_tile_coordinates(
        &mut self,
        start_pos_tile: Point,
        text: &str,
        color: Color,
        options: TextOptions,
        tilesize: i32,
    ) -> DrawResult {
        let start_pos_px = start_pos_tile * tilesize;
        self.draw_text_in_pixel_coordinates(start_pos_px, text, color, options)
    }

    pub fn draw_text_in_pixel_coordinates(
        &mut self,
        start_pos_px: Point,
        text: &str,
        color: Color,
        options: TextOptions,
    ) -> DrawResult {
        // Pull these out to variables prevent mutable borrow conflicts in the `render_line` closure:
        let text_size = self.text_size;
        let tile_size = self.tile_size;
        let mut render_line = |pos_px: Point, line: &str| {
            let mut offset_x = 0;

            // TODO: we need to split this by words or it
            // won't do word breaks, split at punctuation,
            // etc.

            // TODO: also, we're no longer calculating the
            // line height correctly. Needs to be set on the
            // actual result here.
            for chr in line.chars() {
                let (texture_px_x, texture_px_y) =
                    texture_coords_px_from_char(text_size as u32, chr).unwrap_or((0, 0));

                let src = Rectangle::from_point_and_size(
                    Point::new(texture_px_x, texture_px_y),
                    Point::from_i32(text_size),
                );
                let dst = Rectangle::from_point_and_size(
                    pos_px + (offset_x, 0),
                    Point::from_i32(text_size),
                );

                self.drawcalls
                    .push(Drawcall::Image(Texture::Text, src, dst, color));

                let advance_width = glyph_advance_width(text_size as u32, chr).unwrap_or(text_size);
                offset_x += advance_width;
            }
        };

        if options.wrap && options.width > 0 {
            // TODO: handle text alignment for wrapped text
            let lines = wrap_text(text, text_size as u32, options.width, tile_size);
            for (index, line) in lines.iter().skip(options.skip as usize).enumerate() {
                if (index as i32) < options.height {
                    let pos = start_pos_px + Point::new(0, index as i32) * text_size;
                    render_line(pos, line);
                } else {
                    return DrawResult::Overflow;
                }
            }
        } else {
            use crate::engine::TextAlign::*;
            let pos = match options.align {
                Left => start_pos_px,
                Right => {
                    (start_pos_px + Point::new(1, 0) * tile_size)
                        - Point::new(text_width_px(text, text_size as u32, tile_size), 0)
                }
                Center => {
                    let text_width = text_width_px(text, text_size as u32, tile_size);
                    let max_width = options.width * tile_size;
                    if max_width < 1 || (text_width > max_width) {
                        start_pos_px
                    } else {
                        start_pos_px + Point::new((max_width - text_width) / 2, 0)
                    }
                }
            };
            if options.skip == 0 {
                render_line(pos, text);
            } else {
                return DrawResult::Overflow;
            }
        }

        DrawResult::Fit
    }

    pub fn get(&self, pos: Point) -> Color {
        if let Some(ix) = self.index(pos) {
            self.map[ix].background
        } else {
            Default::default()
        }
    }

    pub fn cells(&self) -> impl Iterator<Item = (Point, &Cell)> {
        let padding = self.padding;
        let width = self.full_size_with_padding_in_tiles().x;
        self.map.iter().enumerate().map(move |(index, cell)| {
            let pos = Point::new(index as i32 % width, index as i32 / width);
            let pos = pos - padding;
            (pos, cell)
        })
    }

    pub fn push_drawcalls(&self, drawcalls: &mut Vec<Drawcall>) {
        let offset_px = self.offset_px;
        let display_size_px = self.display_size * self.tile_size;

        if let Some(bg) = self.clear_background_color {
            let full_screen_rect = Rectangle::from_point_and_size(Point::zero(), display_size_px);
            drawcalls.push(Drawcall::Rectangle(full_screen_rect, bg.into()));
        }

        // Render the background tiles separately and before all the other drawcalls.
        for (pos, cell) in self.cells() {
            let (texture, texture_px_x, texture_px_y) =
                match graphic::tilemap_coords_px(self.tile_size as u32, cell.graphic) {
                    Some((tx, ty)) => (Texture::Tilemap, tx, ty),
                    None => {
                        let (tx, ty) =
                            glyph_coords_px_from_char(self.tile_size as u32, cell.graphic.into())
                                .unwrap_or((0, 0));
                        (Texture::Glyph, tx, ty)
                    }
                };
            let texture_size = match texture {
                Texture::Text => self.text_size,
                Texture::Glyph => self.tile_size,
                Texture::Tilemap => TILE_SIZE,
                // NOTE: Egui shouldn't appear in drawcalls, adding it here for completeness
                Texture::Egui => self.tile_size,
            };
            let texture_src = Rectangle::from_point_and_size(
                Point::new(texture_px_x, texture_px_y),
                Point::from_i32(texture_size),
            );
            let background_dst = Rectangle::from_point_and_size(
                Point::new(
                    pos.x * self.tile_size + offset_px.x,
                    pos.y * self.tile_size + offset_px.y,
                ),
                Point::from_i32(self.tile_size),
            );

            let x_offset = match texture {
                // NOTE: We should never get `Texture::Text` in the cells iterator, right?
                Texture::Text => {
                    // NOTE: Center the glyphs in their cells
                    let glyph_width =
                        glyph_advance_width(self.text_size as u32, cell.graphic.into())
                            .unwrap_or(self.tile_size);
                    (self.tile_size as i32 - glyph_width) / 2
                }
                // TODO: we should center this in build.rs and set x_offset to zero
                Texture::Glyph => {
                    // NOTE: Center the glyphs in their cells
                    let glyph_width =
                        glyph_advance_width(self.text_size as u32, cell.graphic.into())
                            .unwrap_or(self.tile_size);
                    (self.tile_size as i32 - glyph_width) / 2
                }
                // NOTE: The graphical tiles are positioned correctly already
                Texture::Tilemap => 0,
                // NOTE: Egui shouldn't appear in drawcalls, adding it here for completeness
                Texture::Egui => 0,
            };
            let glyph_dst = background_dst.offset(Point::new(x_offset, 0));

            if rect_intersects_area(background_dst, display_size_px) {
                drawcalls.push(Drawcall::Rectangle(background_dst, cell.background.into()));
                drawcalls.push(Drawcall::Image(
                    texture,
                    texture_src,
                    glyph_dst,
                    cell.foreground,
                ));
            }
        }

        drawcalls.extend(self.drawcalls.iter());

        if self.fade.alpha > 0 {
            let full_screen_rect = Rectangle::from_point_and_size(Point::zero(), display_size_px);
            drawcalls.push(Drawcall::Rectangle(full_screen_rect, self.fade));
        }
    }
}

/// Returns `true` if the `Rectangle` intersects the area that starts at `(0, 0)`
fn rect_intersects_area(rect: Rectangle, area: Point) -> bool {
    rect.right() >= 0 && rect.left() < area.x && rect.top() < area.y && rect.bottom() >= 0
}

include!(concat!(env!("OUT_DIR"), "/glyph_lookup_table.rs"));
