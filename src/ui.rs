#![allow(dead_code)]

use crate::{
    color::{self, Color},
    engine::{TextOptions, Texture},
    graphic::Graphic,
    palette::Palette,
    point::Point,
};

use egui::{self, widgets, Color32, Context, Rect, Response, RichText, Sense, Ui, Vec2, Widget};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Text<'a> {
    Centered(&'a str),
    Empty,
    EmptySpace(i32),
    Paragraph(&'a str),
    SquareTiles(&'a str),
}

/// Egui container area with no bells and whistles. No border, no
/// elements, no padding, just a regular box you can draw your UI
/// into.
pub fn egui_root(ctx: &Context, rect: Rect, add_contents: impl FnOnce(&mut Ui)) {
    let ui_builder = egui::UiBuilder::new().max_rect(rect);
    let ui = &mut Ui::new(ctx.clone(), egui::Id::new("Root UI Area"), ui_builder);

    add_contents(ui);
}

/// Helper for creating an egui button with the default background and
/// an enabled state.
pub fn button(ui: &mut Ui, text: &str, enabled: bool, palette: &Palette) -> egui::Response {
    sized_button(ui, text, enabled, None, palette)
}

pub fn sized_button(
    ui: &mut Ui,
    text: &str,
    enabled: bool,
    size: Option<Vec2>,
    palette: &Palette,
) -> egui::Response {
    let mut button = egui::Button::new(RichText::new(text).color(palette.gui_text));
    if let Some(size) = size {
        button = button.min_size(size);
    }
    ui.add_enabled(enabled, button)
}

pub fn image_uv_tilesize(texture: Texture, graphic: Graphic, text_size: f32) -> (egui::Rect, f32) {
    let (x, y, tw, th, tilesize) = match texture {
        Texture::Tilemap => {
            let tilesize = crate::graphic::TILE_SIZE as f32;
            let tilemap_width = crate::engine::TILEMAP_TEXTURE_WIDTH as f32;
            let tilemap_height = crate::engine::TILEMAP_TEXTURE_HEIGHT as f32;
            let (x, y) = crate::graphic::tilemap_coords_px(0, graphic).unwrap_or((0, 0));
            (x, y, tilemap_width, tilemap_height, tilesize)
        }
        Texture::Glyph => {
            let tilesize = text_size;
            let tilemap_width = crate::engine::GLYPHMAP_TEXTURE_WIDTH as f32;
            let tilemap_height = crate::engine::GLYPHMAP_TEXTURE_HEIGHT as f32;
            let (x, y) = crate::engine::glyph_coords_px_from_char(tilesize as u32, graphic.into())
                .unwrap_or((0, 0));
            (x, y, tilemap_width, tilemap_height, tilesize)
        }
        texture => {
            log::error!(
                "ERROR: ImageTextButton: unexpected texture type: {:?}",
                texture
            );
            (0, 0, 0.0, 0.0, 0.0)
        }
    };

    let uv = egui::Rect::from_min_size(
        (x as f32 / tw, y as f32 / th).into(),
        Vec2::new(tilesize / tw, tilesize / th),
    );

    (uv, tilesize)
}

/// A clickable button that shows an icon (following the `egui::Image`
/// conventions) followed up by a text.
#[derive(Clone, Debug)]
pub struct ImageTextButton {
    texture: Texture,
    text: String,
    prefix_text: String,
    sense: Sense,
    frame: bool,
    selected: bool,
    graphic: Graphic,
    tile_offset_px: Vec2,
    image_color: Color32,
    text_color: Color32,
    text_disabled_color: Color32,
}

impl ImageTextButton {
    pub fn new(texture: Texture, text: impl Into<String>) -> Self {
        Self {
            texture,
            text: text.into(),
            prefix_text: String::new(),
            sense: Sense::click(),
            frame: true,
            selected: true,
            graphic: Graphic::default(),
            tile_offset_px: Vec2::ZERO,
            image_color: color::WHITE.into(),
            text_color: color::WHITE.into(),
            text_disabled_color: color::WHITE.into(),
        }
    }

    /// Set the optional text that appears before the image.
    pub fn prefix_text(mut self, text: impl Into<String>) -> Self {
        self.prefix_text = text.into();
        self
    }

    /// Set the tile by passing in the `Graphic` enum rather than the
    /// `uv` coordinates.
    pub fn tile(mut self, tile: Graphic) -> Self {
        self.graphic = tile;
        self
    }

    /// Move the tile in the button by this much.
    pub fn tile_offset_px(mut self, offset: impl Into<Vec2>) -> Self {
        self.tile_offset_px = offset.into();
        self
    }

    /// Multiply image color with this. Default is WHITE (no tint).
    pub fn image_color(mut self, color: impl Into<Color32>) -> Self {
        self.image_color = color.into();
        self
    }

    pub fn text_color(mut self, color: impl Into<Color32>) -> Self {
        self.text_color = color.into();
        self
    }

    pub fn text_disabled_color(mut self, color: impl Into<Color32>) -> Self {
        self.text_disabled_color = color.into();
        self
    }

    /// If `true`, mark this button as "selected".
    pub fn selected(mut self, selected: bool) -> Self {
        self.selected = selected;
        self
    }

    /// Turn off the frame
    pub fn frame(mut self, frame: bool) -> Self {
        self.frame = frame;
        self
    }

    /// By default, buttons senses clicks.
    /// Change this to a drag-button with `Sense::drag()`.
    pub fn sense(mut self, sense: Sense) -> Self {
        self.sense = sense;
        self
    }
}

impl Widget for ImageTextButton {
    fn ui(self, ui: &mut Ui) -> Response {
        let Self {
            texture,
            text,
            prefix_text,
            sense,
            frame,
            selected,
            graphic,
            tile_offset_px,
            image_color,
            text_color,
            text_disabled_color,
        } = self;

        let font_size = ui.ctx().style().text_styles[&egui::TextStyle::Button].size;
        let font_id = egui::FontId::monospace(font_size);
        let prefix_galley =
            ui.fonts(|reader| reader.layout_no_wrap(prefix_text, font_id.clone(), text_color));
        let text_galley =
            ui.fonts(|reader| reader.layout_no_wrap(text, font_id.clone(), text_color));

        let (uv, _tilesize) = image_uv_tilesize(texture, graphic, text_galley.rect.height());

        let sized_texture =
            egui::load::SizedTexture::new(texture, Vec2::splat(text_galley.rect.height()));
        let image = widgets::Image::new(sized_texture).uv(uv).tint(image_color);

        let button_padding = ui.spacing().button_padding;
        let size = Vec2::new(
            prefix_galley.rect.width() + button_padding.x,
            button_padding.y,
        ) + (image.size().unwrap_or(Vec2::ZERO) + 3.0 * button_padding)
            + Vec2::new(text_galley.rect.width(), 0.0);
        let (rect, response) = ui.allocate_exact_size(size, sense);
        let text_pos = rect.min
            + Vec2::new(prefix_galley.rect.width(), 0.0)
            + Vec2::new(
                image.size().unwrap_or(Vec2::ZERO).x + button_padding.x * 2.0,
                button_padding.y,
            );

        let prefix_translate = Vec2::new(
            prefix_galley.rect.width() + tile_offset_px.x,
            tile_offset_px.y,
        );

        if ui.clip_rect().intersects(rect) {
            let visuals = ui.style().interact(&response);

            let painter = ui.painter();

            if selected {
                painter.rect(
                    rect,
                    visuals.corner_radius,
                    visuals.bg_fill,
                    visuals.bg_stroke,
                    egui::StrokeKind::Inside,
                );
                painter.galley(
                    rect.min + button_padding,
                    prefix_galley,
                    Color32::PLACEHOLDER,
                );
                painter.galley(text_pos, text_galley, Color32::PLACEHOLDER);
            } else if frame {
                painter.rect(
                    rect.expand(visuals.expansion),
                    visuals.corner_radius,
                    visuals.bg_fill,
                    visuals.bg_stroke,
                    egui::StrokeKind::Inside,
                );
                painter.galley_with_override_text_color(
                    rect.min + button_padding,
                    prefix_galley,
                    text_disabled_color,
                );
                painter.galley_with_override_text_color(text_pos, text_galley, text_disabled_color);
            }

            let image_rect = ui.layout().align_size_within_rect(
                image.size().unwrap_or(Vec2::ZERO),
                rect.shrink2(button_padding).translate(prefix_translate),
            );
            image.bg_fill(visuals.bg_fill).paint_at(ui, image_rect);
        }

        response
    }
}

pub fn progress_bar(
    ui: &mut Ui,
    bg_cmd_index: egui::layers::ShapeIdx,
    fg_cmd_index: egui::layers::ShapeIdx,
    top_left: egui::Pos2,
    width: f32,
    height: f32,
    percent: f32,
    bg_color: Color,
    fg_color: Color,
) {
    use egui::epaint::Shape;

    let percent = percent.clamp(0.0, 1.0);
    let background_rect = Shape::rect_filled(
        Rect::from_min_size(top_left, [width, height].into()),
        0.0,
        bg_color,
    );

    let foreground_rect = Shape::rect_filled(
        Rect::from_min_size(top_left, [width * percent, height].into()),
        0.0,
        fg_color,
    );

    ui.painter().set(bg_cmd_index, background_rect);

    if percent > 0.0 {
        ui.painter().set(fg_cmd_index, foreground_rect);
    }
}

#[derive(Clone, Default)]
pub struct Button {
    pub pos: Point,
    pub text: String,
    pub color: Color,
    pub text_options: TextOptions,
}

impl Button {
    pub fn new(pos: Point, text: &str, palette: &Palette) -> Self {
        Button {
            pos,
            text: text.into(),
            color: palette.gui_text,
            ..Default::default()
        }
    }

    pub fn color(self, color: Color) -> Self {
        Button { color, ..self }
    }

    pub fn align_left(self) -> Self {
        Button {
            text_options: TextOptions::align_left(),
            ..self
        }
    }

    pub fn align_right(self) -> Self {
        Button {
            text_options: TextOptions::align_right(),
            ..self
        }
    }

    pub fn align_center(self, width: i32) -> Self {
        Button {
            text_options: TextOptions::align_center(width),
            ..self
        }
    }
}
