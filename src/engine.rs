#![allow(dead_code)]

use crate::{
    color::{self, Color, ColorAlpha},
    graphic::{self, Graphic, TILE_SIZE},
    point::Point,
    rect::Rectangle,
};

use std::fmt;

use egui::{ClippedPrimitive, TextureId};
use serde::{Deserialize, Serialize};

#[cfg(any(feature = "glutin-backend", feature = "sdl3-backend"))]
mod loop_state;

#[cfg(feature = "glutin-backend")]
pub mod glutin;

#[cfg(any(feature = "glutin-backend", feature = "sdl3-backend"))]
pub mod opengl;

#[cfg(feature = "sdl3-backend")]
pub mod sdl3;

#[cfg(feature = "sdl2-backend")]
pub mod sdl2;

pub mod headless;

pub const DRAWCALL_CAPACITY: usize = 8000;
pub const VERTEX_CAPACITY: usize = 70_000;
pub const VERTEX_COMPONENT_COUNT: usize = 9;
const VERTEX_BUFFER_CAPACITY: usize = VERTEX_COMPONENT_COUNT * VERTEX_CAPACITY;

const TEXTURE_EGUI: u64 = 0;
// TODO: update the numbers here
const TEXTURE_GLYPH: u64 = 1;
const TEXTURE_TILEMAP: u64 = 2;

#[derive(Copy, Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[repr(u64)]
pub enum Texture {
    Egui = TEXTURE_EGUI,
    Glyph = TEXTURE_GLYPH,
    Tilemap = TEXTURE_TILEMAP,
}

impl From<Texture> for f32 {
    fn from(tex: Texture) -> f32 {
        tex as u64 as f32
    }
}

impl From<Texture> for TextureId {
    fn from(tex: Texture) -> TextureId {
        match tex {
            Texture::Egui => TextureId::Managed(0), // NOTE: 0 is the default egui font texture
            Texture::Glyph => TextureId::User(tex as u64),
            Texture::Tilemap => TextureId::User(tex as u64),
        }
    }
}

/// Visual style of the game.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum VisualStyle {
    /// Graphical tiles
    Graphical,
    /// Textual glyphs (classic roguelike ASCII visuals)
    Textual,
}

// Define the constants for the string variants to prevent typos in the code.
pub const VISUAL_STYLE_GRAPHICAL_STR: &str = "graphical";
pub const VISUAL_STYLE_TEXTUAL_STR: &str = "textual";

impl fmt::Display for VisualStyle {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
        use self::VisualStyle::*;
        let s = match *self {
            Graphical => VISUAL_STYLE_GRAPHICAL_STR,
            Textual => VISUAL_STYLE_TEXTUAL_STR,
        };
        f.write_str(s)
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
    fn to_f32_array(self) -> [f32; 9] {
        [
            self.texture_id,
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

impl From<ColorAlpha> for [f32; 4] {
    fn from(coloralpha: ColorAlpha) -> [f32; 4] {
        [
            f32::from(coloralpha.rgb.r) / 255.0,
            f32::from(coloralpha.rgb.g) / 255.0,
            f32::from(coloralpha.rgb.b) / 255.0,
            f32::from(coloralpha.alpha) / 255.0,
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
        self.extend(vertex.to_f32_array())
    }

    fn count(&self) -> usize {
        assert_eq!(self.len() % 9, 0);
        self.len() / 9
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
            // NOTE: (Rectangle, ColorAlpha)
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

// NOTE: convert Egui indexed vertices into ones our
// engine understands. I.e. naive 3 vertices per
// triangle with duplication.
fn drawcalls_from_egui(
    opengl_app: &opengl::OpenGlApp,
    ui_paint_batches: &Vec<ClippedPrimitive>,
) -> (Vec<Vertex>, Vec<([f32; 4], i32, i32)>) {
    use egui::epaint::Primitive;

    // TODO: consider doing updating our engine to suport
    // vertex indices.
    let mut ui_vertices = vec![];
    let mut batches = vec![];
    let mut index = 0;

    for clipped_primitive in ui_paint_batches {
        let ClippedPrimitive {
            clip_rect,
            primitive,
        } = clipped_primitive;

        if let Primitive::Mesh(mesh) = primitive {
            let texture_id = match mesh.texture_id {
                egui::TextureId::Managed(0) => Texture::Egui.into(),
                egui::TextureId::Managed(id) => {
                    log::error!("Unexpected Managed texture ID: {}", id);
                    Texture::Egui.into()
                }
                egui::TextureId::User(id) => id as f32,
            };

            // NOTE: the shader expects the egui texture (uv)
            // coordinates to be normalised, but everything
            // else expects pixel coordinates.
            //
            // However, everything that comes out of egui *is
            // going to be normalised* so we need to
            // "denormalise" it by multiplying the uv coords
            // with the size of the texture in pixels.
            //
            // For egui we just multiply by 1.0 which has no
            // effect.
            let texture_size = match mesh.texture_id {
                egui::TextureId::Managed(0) => [1.0, 1.0],
                egui::TextureId::Managed(id) => {
                    log::error!(
                        "Unexpected TextureId::Managed({})! We should only ever see ID of 0",
                        id
                    );
                    [1.0, 1.0]
                }
                egui::TextureId::User(TEXTURE_GLYPH) => opengl_app.glyphmap_size_px,
                egui::TextureId::User(TEXTURE_TILEMAP) => opengl_app.tilemap_size_px,
                id => {
                    log::error!(
                        "ERROR[Winit RedrawRequested]: unknown texture ID: `{:?}`",
                        id
                    );
                    [1.0, 1.0]
                }
            };

            for &index in &mesh.indices {
                let egui_vertex = match mesh.vertices.get(index as usize) {
                    Some(vertex) => vertex,
                    None => {
                        log::error!("Can't index into the mesh.vertices");
                        continue;
                    }
                };
                let color = Color {
                    r: egui_vertex.color.r(),
                    g: egui_vertex.color.g(),
                    b: egui_vertex.color.b(),
                }
                .alpha(egui_vertex.color.a());
                let (u, v) = (egui_vertex.uv.x, egui_vertex.uv.y);

                let pos = egui_vertex.pos;
                let vertex = Vertex {
                    texture_id,
                    pos_px: [pos.x, pos.y],
                    tile_pos: [u * texture_size[0], v * texture_size[1]],
                    color: color.into(),
                };
                ui_vertices.push(vertex);
            }

            let vertex_count = mesh.indices.len() as i32;
            batches.push((
                [
                    clip_rect.left_top().x,
                    clip_rect.left_top().y,
                    clip_rect.right_bottom().x,
                    clip_rect.right_bottom().y,
                ],
                index,
                vertex_count,
            ));
            index += vertex_count;
        }
    }

    (ui_vertices, batches)
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
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

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum TextAlign {
    Left,
    Right,
    Center,
}

pub trait TextMetrics {
    fn tile_width_px(&self) -> i32;

    fn text_width_px(&self) -> i32;
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
    /// Position of the mouse on screen in tiles. The top-left corner is (0, 0), grows down and to the right.
    pub tile_pos: Point,
    /// Position of the mouse on screen in pixels. The top-left corner is (0, 0), grows down and to the right.
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
    pub background_graphic: Graphic,
    pub foreground_graphic: Graphic,
    pub foreground: Color,
    pub empty_color: Color,
    pub background_color: Color,
    pub offset_px: Point,
}

impl Default for Cell {
    fn default() -> Self {
        Cell {
            background_graphic: Graphic::Empty,
            foreground_graphic: Graphic::Empty,
            foreground: Color { r: 0, g: 0, b: 0 },
            empty_color: Color { r: 0, g: 0, b: 0 },
            background_color: Color { r: 0, g: 0, b: 0 },
            offset_px: Point::zero(),
        }
    }
}

#[derive(Copy, Clone, Debug)]
pub struct OffsetTile {
    pub pos: Point,
    pub graphic: Graphic,
    pub color: Color,
    pub offset_px: Point,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
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
    pub offset_tiles: Vec<OffsetTile>,
    drawcalls: Vec<Drawcall>,
    pub fade: ColorAlpha,
    clear_background_color: Option<Color>,
}

#[allow(dead_code)]
impl Display {
    /// Create a new Display.
    ///
    /// `screen_size_px`: dimension of the game window in logical pixels.
    /// `tilesize`: size (in pixels) of a single tile. Tiles are square.
    pub fn new(screen_size_px: Point, tile_size: i32, text_size: i32) -> Self {
        assert!(screen_size_px > Point::zero());
        assert!(tile_size > 0);
        assert!(text_size > 0);
        // NOTE: this padding is only here to make the screen scroll smoothly.
        // Without it, the partial tiles would not appear.
        let display_size = {
            let size = screen_size_px / tile_size;
            let extra_x = i32::from(size.x * tile_size < screen_size_px.x);
            let extra_y = i32::from(size.y * tile_size < screen_size_px.y);
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
            offset_tiles: vec![],
            drawcalls: Vec::with_capacity(DRAWCALL_CAPACITY),
            fade: color::INVISIBLE,
            ..Default::default()
        }
    }

    pub fn clear(&mut self, empty_color: Color) {
        for cell in &mut self.map {
            *cell = Cell {
                empty_color,
                ..Default::default()
            };
        }
        self.drawcalls.clear();
        self.fade = color::INVISIBLE;
        self.clear_background_color = Some(empty_color);
        self.offset_tiles.clear();
    }

    /// This is the full display size: game plus padding.
    pub fn full_size_with_padding_in_tiles(&self) -> Point {
        self.display_size + (self.padding * 2)
    }

    /// This is the size of the display as originally requested.
    ///
    /// There is no padding here -- this should correspond to what's
    /// shown on the screen when there is no scrolling or other
    /// shenanigans going on.
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

    pub fn set_cell(
        &mut self,
        pos: Point,
        graphic: Graphic,
        graphic_color: Color,
        empty_color: Color,
    ) {
        if let Some(ix) = self.index(pos)
            && let Some(entry) = self.map.get_mut(ix)
        {
            *entry = Cell {
                background_graphic: graphic,
                foreground_graphic: graphic,
                empty_color,
                foreground: graphic_color,
                background_color: graphic_color,
                offset_px: Point::zero(),
            };
        }
    }

    pub fn set_foreground_graphic(
        &mut self,
        pos: Point,
        foreground_graphic: Graphic,
        foreground: Color,
    ) {
        if let Some(ix) = self.index(pos)
            && let Some(entry) = self.map.get_mut(ix)
        {
            entry.foreground_graphic = foreground_graphic;
            entry.foreground = foreground;
        }
    }

    pub fn set_empty_color(&mut self, pos: Point, empty_color: Color) {
        if let Some(ix) = self.index(pos)
            && let Some(entry) = self.map.get_mut(ix)
        {
            entry.empty_color = empty_color;
        }
    }

    pub fn set_offset(&mut self, pos: Point, offset_px: Point) {
        if let Some(ix) = self.index(pos)
            && let Some(entry) = self.map.get_mut(ix)
        {
            entry.offset_px = offset_px;
        }
    }

    pub fn push_fg_to_bg(&mut self, pos: Point) {
        if let Some(ix) = self.index(pos)
            && let Some(cell) = self.map.get_mut(ix)
        {
            cell.background_graphic = cell.foreground_graphic;
            cell.background_color = cell.foreground;
        }
    }

    /// Set the value (RGBA) to fade the screen with.
    /// Unlike alpha, the `fade` argument is inverted: 1.0 means no fade, 0.0 means fully faded.
    pub fn set_fade(&mut self, color: Color, fade: f32) {
        let fade = fade.clamp(0.0, 1.0);
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

    pub fn get(&self, pos: Point) -> Color {
        if let Some(ix) = self.index(pos) {
            self.map
                .get(ix)
                .map(|cell| cell.empty_color)
                .unwrap_or_default()
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

    pub fn push_drawcalls(&self, visual_style: VisualStyle, drawcalls: &mut Vec<Drawcall>) {
        let display_offset_px = self.offset_px;
        let display_size_px = self.display_size * self.tile_size;

        if let Some(bg) = self.clear_background_color {
            let full_screen_rect = Rectangle::from_point_and_size(Point::zero(), display_size_px);
            drawcalls.push(Drawcall::Rectangle(full_screen_rect, bg.into()));
        }

        let mut deferred_drawcalls = vec![];

        // Render the background tiles separately and before all the other drawcalls.
        for (pos, cell) in self.cells() {
            let (fg_texture, fg_texture_px_x, fg_texture_px_y) = match visual_style {
                VisualStyle::Graphical => {
                    match graphic::tilemap_coords_px(self.tile_size as u32, cell.foreground_graphic)
                    {
                        Some((tx, ty)) => (Texture::Tilemap, tx, ty),
                        // NOTE: Fall back to glyphs if the graphic coordinates can't be provided:
                        None => {
                            let (tx, ty) = glyph_coords_px_from_char(
                                self.tile_size as u32,
                                cell.foreground_graphic.into(),
                            )
                            .unwrap_or((0, 0));
                            (Texture::Glyph, tx, ty)
                        }
                    }
                }
                VisualStyle::Textual => {
                    let (tx, ty) = glyph_coords_px_from_char(
                        self.tile_size as u32,
                        cell.foreground_graphic.into(),
                    )
                    .unwrap_or((0, 0));
                    (Texture::Glyph, tx, ty)
                }
            };

            let texture_size = match fg_texture {
                Texture::Glyph => self.tile_size,
                Texture::Tilemap => TILE_SIZE,
                // NOTE: Egui shouldn't appear in drawcalls, adding it here for completeness
                Texture::Egui => self.tile_size,
            };
            let fg_texture_src = Rectangle::from_point_and_size(
                Point::new(fg_texture_px_x, fg_texture_px_y),
                Point::from_i32(texture_size),
            );
            let background_dst = Rectangle::from_point_and_size(
                Point::new(
                    pos.x * self.tile_size + display_offset_px.x,
                    pos.y * self.tile_size + display_offset_px.y,
                ),
                Point::from_i32(self.tile_size),
            );

            let (bg_texture, bg_texture_px_x, bg_texture_px_y) = match visual_style {
                VisualStyle::Graphical => {
                    match graphic::tilemap_coords_px(self.tile_size as u32, cell.background_graphic)
                    {
                        Some((tx, ty)) => (Texture::Tilemap, tx, ty),
                        // NOTE: Fall back to glyphs if the graphic coordinates can't be provided:
                        None => {
                            let (tx, ty) = glyph_coords_px_from_char(
                                self.tile_size as u32,
                                cell.background_graphic.into(),
                            )
                            .unwrap_or((0, 0));
                            (Texture::Glyph, tx, ty)
                        }
                    }
                }
                VisualStyle::Textual => {
                    let (tx, ty) = glyph_coords_px_from_char(
                        self.tile_size as u32,
                        cell.background_graphic.into(),
                    )
                    .unwrap_or((0, 0));
                    (Texture::Glyph, tx, ty)
                }
            };
            let bg_texture_src = Rectangle::from_point_and_size(
                Point::new(bg_texture_px_x, bg_texture_px_y),
                Point::from_i32(texture_size),
            );

            // NOTE: Only render areas within the display size
            if rect_intersects_area(background_dst, display_size_px) {
                drawcalls.push(Drawcall::Rectangle(background_dst, cell.empty_color.into()));
                let bg_image = Drawcall::Image(
                    bg_texture,
                    bg_texture_src,
                    background_dst,
                    cell.background_color,
                );
                drawcalls.push(bg_image);
                let image = Drawcall::Image(
                    fg_texture,
                    fg_texture_src,
                    background_dst.offset(cell.offset_px),
                    cell.foreground,
                );
                if cell.offset_px.is_zero() {
                    drawcalls.push(image);
                } else {
                    // NOTE: we can't render the image now because it
                    // would be overwritten by all the other images on the grid.
                    //
                    // So everything that's got an offset to it will
                    // be deferred and only rendered after all the
                    // grid-aligned tiles are rendered.
                    //
                    // This also means all the offset cells are
                    // rendered on top of the non-offset ones.
                    //
                    // If there are two cells with an offset that
                    // overlap, they will of course overlap on screen
                    // too.
                    deferred_drawcalls.push(image);
                }
            }
        }

        for tile in &self.offset_tiles {
            let (texture, texture_px_x, texture_px_y) = match visual_style {
                VisualStyle::Graphical => {
                    // TODO: handle background graphics here too!
                    match graphic::tilemap_coords_px(self.tile_size as u32, tile.graphic) {
                        Some((tx, ty)) => (Texture::Tilemap, tx, ty),
                        // NOTE: Fall back to glyphs if the graphic coordinates can't be provided:
                        None => {
                            let (tx, ty) = glyph_coords_px_from_char(
                                self.tile_size as u32,
                                tile.graphic.into(),
                            )
                            .unwrap_or((0, 0));
                            (Texture::Glyph, tx, ty)
                        }
                    }
                }
                VisualStyle::Textual => {
                    let (tx, ty) =
                        glyph_coords_px_from_char(self.tile_size as u32, tile.graphic.into())
                            .unwrap_or((0, 0));
                    (Texture::Glyph, tx, ty)
                }
            };

            let texture_size = match texture {
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
                    tile.pos.x * self.tile_size + display_offset_px.x,
                    tile.pos.y * self.tile_size + display_offset_px.y,
                ),
                Point::from_i32(self.tile_size),
            );
            let image = Drawcall::Image(
                texture,
                texture_src,
                background_dst.offset(tile.offset_px),
                tile.color,
            );
            drawcalls.push(image);
        }

        drawcalls.extend(deferred_drawcalls);
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
