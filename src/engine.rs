#![allow(dead_code)]

use crate::{
    color::{self, Color, ColorAlpha},
    game::RunningState,
    keys::Key,
    point::Point,
    rect::Rectangle,
    settings::{Settings, Store as SettingsStore},
    state::State,
    ui::Button,
    util,
};

use std::time::Duration;

use serde::{Deserialize, Serialize};

#[cfg(feature = "glutin-backend")]
pub mod glutin;

#[cfg(any(feature = "glutin-backend", feature = "sdl-backend"))]
pub mod opengl;

#[cfg(feature = "sdl-backend")]
pub mod sdl;

#[cfg(feature = "remote")]
pub mod remote;

#[cfg(feature = "web")]
pub mod wasm;

pub const DRAWCALL_CAPACITY: usize = 8000;
pub const VERTEX_CAPACITY: usize = 50_000;
pub const VERTEX_COMPONENT_COUNT: usize = 8;
const VERTEX_BUFFER_CAPACITY: usize = VERTEX_COMPONENT_COUNT * VERTEX_CAPACITY;

/// The drawcalls that the engine will process and render.
#[derive(Copy, Clone, Debug, Serialize, Deserialize)]
pub enum Drawcall {
    Rectangle(Rectangle, ColorAlpha),
    Image(Rectangle, Rectangle, Color),
}

#[derive(Copy, Clone, Debug)]
#[repr(C)]
pub struct Vertex {
    /// Position in the tile coordinates.
    ///
    /// Note that this doesn't have to be an integer, so you can
    /// implement smooth positioning by using a fractional value.
    pub pos_px: [f32; 2],

    /// Pixel position in the tile map / sprite sheet.
    ///
    /// If both values are negative, no texture will be rendered.
    /// Instead, the colour will be used to fill the background.
    /// That's how render rectangles.
    pub tile_pos_px: [f32; 2],

    /// Colour of the glyph. The glyphs are greyscale, so this is how
    /// we set the final colour.
    pub color: [f32; 4],
}

impl Vertex {
    #[allow(dead_code)]
    fn to_f32_array(&self) -> [f32; 8] {
        [
            self.pos_px[0],
            self.pos_px[1],
            self.tile_pos_px[0],
            self.tile_pos_px[1],
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
}

impl VertexStore for Vec<Vertex> {
    fn push(&mut self, vertex: Vertex) {
        self.push(vertex)
    }
}

impl VertexStore for Vec<f32> {
    fn push(&mut self, vertex: Vertex) {
        self.extend(&vertex.to_f32_array())
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

                let tile_pos_px = [-1.0, -1.0];
                let color = (*color).into();

                vertices.push(Vertex {
                    pos_px: [pos_x, pos_y],
                    tile_pos_px,
                    color,
                });
                vertices.push(Vertex {
                    pos_px: [pos_x + dim_x, pos_y],
                    tile_pos_px,
                    color,
                });
                vertices.push(Vertex {
                    pos_px: [pos_x, pos_y + dim_y],
                    tile_pos_px,
                    color,
                });

                vertices.push(Vertex {
                    pos_px: [pos_x + dim_x, pos_y],
                    tile_pos_px,
                    color,
                });
                vertices.push(Vertex {
                    pos_px: [pos_x, pos_y + dim_y],
                    tile_pos_px,
                    color,
                });
                vertices.push(Vertex {
                    pos_px: [pos_x + dim_x, pos_y + dim_y],
                    tile_pos_px,
                    color,
                });
            }

            // NOTE: (Rectangle, Rectangle, Color)
            Drawcall::Image(src, dst, color) => {
                let pixel_pos = dst.top_left();
                let (pos_x, pos_y) = (pixel_pos.x as f32, pixel_pos.y as f32);
                let (tile_width, tile_height) = (dst.width() as f32, dst.height() as f32);
                let (tilemap_x, tilemap_y) = (src.top_left().x as f32, src.top_left().y as f32);

                // NOTE: cut off the area that's not in the display.
                //
                // Any drawcalls that would be entirely invisible will
                // have been filtered by now. But we still need to
                // process drawcalls that are only partially visible.
                // So this fixes the rendered position and height to
                // do just that.
                let (pos_x, tilemap_x, tile_width) = if pos_x < 0.0 {
                    (0.0, tilemap_x + (0.0 - pos_x), tile_width + pos_x)
                } else if pos_x + tile_width >= display_size_x {
                    (pos_x, tilemap_x, display_size_x - pos_x)
                } else {
                    (pos_x, tilemap_x, tile_width)
                };
                let (pos_y, tilemap_y, tile_height) = if pos_y < 0.0 {
                    (0.0, tilemap_y + (0.0 - pos_y), tile_height + pos_y)
                } else if pos_y + tile_height >= display_size_y {
                    (pos_y, tilemap_y, display_size_y - pos_y)
                } else {
                    (pos_y, tilemap_y, tile_height)
                };

                let rgba: ColorAlpha = (*color).into();
                let color = rgba.into();

                // NOTE: draw the glyph
                vertices.push(Vertex {
                    pos_px: [pos_x, pos_y],
                    tile_pos_px: [tilemap_x, tilemap_y],
                    color,
                });
                vertices.push(Vertex {
                    pos_px: [pos_x + tile_width, pos_y],
                    tile_pos_px: [tilemap_x + tile_width, tilemap_y],
                    color,
                });
                vertices.push(Vertex {
                    pos_px: [pos_x, pos_y + tile_height],
                    tile_pos_px: [tilemap_x, tilemap_y + tile_height],
                    color,
                });

                vertices.push(Vertex {
                    pos_px: [pos_x + tile_width, pos_y],
                    tile_pos_px: [tilemap_x + tile_width, tilemap_y],
                    color,
                });
                vertices.push(Vertex {
                    pos_px: [pos_x, pos_y + tile_height],
                    tile_pos_px: [tilemap_x, tilemap_y + tile_height],
                    color,
                });
                vertices.push(Vertex {
                    pos_px: [pos_x + tile_width, pos_y + tile_height],
                    tile_pos_px: [tilemap_x + tile_width, tilemap_y + tile_height],
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

    pub height: i32,
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

    /// Return the height in tiles of the given text.
    ///
    /// Panics when `text_drawcall` is not `Draw::Text`
    fn get_text_height(&self, text: &str, options: TextOptions) -> i32 {
        if options.wrap && options.width > 0 {
            let font_size = self.tile_width_px() as u32;
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
        let font_size = self.tile_width_px() as u32;
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
        self.text_rect(button.pos, &button.text, button.text_options)
    }
}

// Calculate the width in pixels of a given text
fn text_width_px(text: &str, font_size: u32, tile_width_px: i32) -> i32 {
    text.chars()
        .map(|chr| glyph_advance_width(font_size, chr).unwrap_or(tile_width_px as i32))
        .sum()
}

fn wrap_text(text: &str, font_size: u32, width_tiles: i32, tile_width_px: i32) -> Vec<String> {
    let mut result = vec![];
    let wrap_width_px = width_tiles * tile_width_px;
    let space_width = glyph_advance_width(font_size, ' ').unwrap_or(tile_width_px as i32);
    let font_size = tile_width_px as u32;

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
    native_display_px: [f32; 2],
    window_size_px: [f32; 2],
    display_px: [f32; 2],
    extra_px: [f32; 2],
}

/// Calculate the dimensions to provide the largest display
/// area while maintaining the aspect ratio (and letterbox the
/// display).
fn calculate_display_info(
    window_size_px: [f32; 2],
    display_size_tiles: Point,
    tilesize_px: i32,
) -> DisplayInfo {
    let window_width = window_size_px[0] as f32;
    let window_height = window_size_px[1] as f32;
    let tilecount_x = display_size_tiles.x as f32;
    let tilecount_y = display_size_tiles.y as f32;
    let tilesize = tilesize_px as f32;

    let unscaled_game_width = tilecount_x * tilesize;
    let unscaled_game_height = tilecount_y * tilesize;

    // TODO: we're assuming that the unscaled dimensions
    // already fit into the display. So the game is only going
    // to be scaled up, not down.

    // NOTE: try if the hight should fill the display area
    let scaled_tilesize = (window_height / tilecount_y).floor();
    let scaled_width = scaled_tilesize * tilecount_x;
    let scaled_height = scaled_tilesize * tilecount_y;
    let (final_scaled_width, final_scaled_height) = if scaled_width <= window_width {
        (scaled_width, scaled_height)
    } else {
        // NOTE: try if the width should fill the display area
        let scaled_tilesize = (window_width / tilecount_x).floor();
        let scaled_width = scaled_tilesize * tilecount_x;
        let scaled_height = scaled_tilesize * tilecount_y;

        if scaled_height <= window_height {
            // NOTE: we're good
        } else {
            log::error!("Can't scale neither to width nor height wtf.");
        }
        (scaled_width, scaled_height)
    };

    let native_display_px = [unscaled_game_width, unscaled_game_height];
    let display_px = [final_scaled_width, final_scaled_height];
    let extra_px = [
        window_width - final_scaled_width,
        window_height - final_scaled_height,
    ];

    DisplayInfo {
        native_display_px,
        window_size_px,
        display_px,
        extra_px,
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
}

impl Default for Cell {
    fn default() -> Self {
        Cell {
            glyph: ' ',
            foreground: Color { r: 0, g: 0, b: 0 },
            background: Color {
                r: 255,
                g: 0,
                b: 255,
            },
        }
    }
}

#[derive(Default)]
pub struct Display {
    display_size: Point,
    pub tilesize: i32,
    pub offset_px: Point,
    padding: Point,
    map: Vec<Cell>,
    drawcalls: Vec<Drawcall>,
    pub fade: ColorAlpha,
    clear_background_color: Option<Color>,
}

#[allow(dead_code)]
impl Display {
    pub fn new(display_size: Point, padding: Point, tilesize: i32) -> Self {
        assert!(display_size > Point::zero());
        assert!(padding >= Point::zero());
        assert!(tilesize > 0);
        let size = display_size + (padding * 2);
        Display {
            display_size,
            padding,
            tilesize,
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

    pub fn set(&mut self, pos: Point, glyph: char, foreground: Color, background: Color) {
        if let Some(ix) = self.index(pos) {
            self.map[ix] = Cell {
                glyph,
                foreground,
                background,
            };
        }
    }

    pub fn set_glyph(&mut self, pos: Point, glyph: char, foreground: Color) {
        if let Some(ix) = self.index(pos) {
            self.map[ix].glyph = glyph;
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
        let top_left_px = rect.top_left() * self.tilesize;
        let dimensions_px = rect.size() * self.tilesize;

        let rect = Rectangle::from_point_and_size(top_left_px, dimensions_px);
        self.drawcalls.push(Drawcall::Rectangle(rect, color.into()));
    }

    /// Draw a Button
    pub fn draw_button(&mut self, button: &Button) {
        self.draw_text(button.pos, &button.text, button.color, button.text_options);
    }

    pub fn draw_text(&mut self, start_pos: Point, text: &str, color: Color, options: TextOptions) {
        let tilesize = self.tilesize;
        let font_size = tilesize as u32;
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
                    texture_coords_px_from_char(font_size, chr).unwrap_or((0, 0));

                let src = Rectangle::from_point_and_size(
                    Point::new(texture_px_x, texture_px_y),
                    Point::from_i32(self.tilesize),
                );
                let dst = Rectangle::from_point_and_size(
                    pos_px + (offset_x, 0),
                    Point::from_i32(self.tilesize),
                );

                self.drawcalls.push(Drawcall::Image(src, dst, color));

                let advance_width = glyph_advance_width(font_size, chr).unwrap_or(self.tilesize);
                offset_x += advance_width;
            }
        };

        if options.wrap && options.width > 0 {
            // TODO: handle text alignment for wrapped text
            let lines = wrap_text(text, font_size, options.width, tilesize);
            for (index, line) in lines.iter().enumerate() {
                let pos = (start_pos + Point::new(0, index as i32)) * tilesize;
                render_line(pos, line);
            }
        } else {
            use crate::engine::TextAlign::*;
            let pos = match options.align {
                Left => start_pos * tilesize,
                Right => {
                    (start_pos + (1, 0)) * tilesize
                        - Point::new(text_width_px(text, font_size, tilesize), 0)
                }
                Center => {
                    let text_width = text_width_px(text, font_size, tilesize);
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

    pub fn cells(&self) -> impl Iterator<Item = (Point, &Cell)> {
        let padding = self.padding;
        let width = self.size().x;
        self.map.iter().enumerate().map(move |(index, cell)| {
            let pos = Point::new(index as i32 % width, index as i32 / width);
            let pos = pos - padding;
            (pos, cell)
        })
    }

    pub fn push_drawcalls(&self, drawcalls: &mut Vec<Drawcall>) {
        let offset_px = self.offset_px;
        let tilesize = self.tilesize;
        let font_size = tilesize as u32;
        let display_size_px = self.display_size * tilesize;

        if let Some(bg) = self.clear_background_color {
            let full_screen_rect = Rectangle::from_point_and_size(Point::zero(), display_size_px);
            drawcalls.push(Drawcall::Rectangle(full_screen_rect, bg.into()));
        }

        // Render the background tiles separately and before all the other drawcalls.
        for (pos, cell) in self.cells() {
            let (texture_px_x, texture_px_y) =
                texture_coords_px_from_char(font_size, cell.glyph).unwrap_or((0, 0));
            let texture_src = Rectangle::from_point_and_size(
                Point::new(texture_px_x, texture_px_y),
                Point::from_i32(tilesize),
            );
            let background_dst = Rectangle::from_point_and_size(
                Point::new(
                    pos.x * tilesize + offset_px.x,
                    pos.y * tilesize + offset_px.y,
                ),
                Point::from_i32(tilesize),
            );

            // NOTE: Center the glyphs in their cells
            let glyph_width = glyph_advance_width(font_size, cell.glyph).unwrap_or(tilesize);
            let x_offset = (tilesize as i32 - glyph_width) / 2;
            let glyph_dst = background_dst.offset(Point::new(x_offset, 0));

            if rect_intersects_area(background_dst, display_size_px) {
                drawcalls.push(Drawcall::Rectangle(background_dst, cell.background.into()));
                drawcalls.push(Drawcall::Image(texture_src, glyph_dst, cell.foreground));
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

#[allow(dead_code)]
pub type UpdateFn = fn(
    &mut State,
    dt: Duration,
    size: Point,
    fps: i32,
    keys: &[Key],
    mouse: Mouse,
    settings: &mut Settings,
    metrics: &dyn TextMetrics,
    settings_store: &mut SettingsStore,
    display: &mut Display,
) -> RunningState;

include!(concat!(env!("OUT_DIR"), "/glyph_lookup_table.rs"));
