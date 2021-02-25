#![allow(dead_code)]

use crate::{
    color::{self, Color},
    engine::{Display, DrawResult, TextMetrics, TextOptions, Texture},
    graphic::{self, Graphic},
    palette::Palette,
    point::Point,
    rect::Rectangle,
};

use egui::{self, widgets, Color32, Rect, Response, Sense, Ui, Vec2, Widget};

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Text<'a> {
    Centered(&'a str),
    Empty,
    EmptySpace(i32),
    Paragraph(&'a str),
    SquareTiles(&'a str),
}

pub fn render_text_flow(
    text_flow: &[Text<'_>],
    rect: Rectangle,
    starting_line: i32,
    metrics: &dyn TextMetrics,
    display: &mut Display,
    palette: &Palette,
) -> DrawResult {
    use self::Text::*;

    let mut skip = starting_line;
    let mut ypos_px = 0;
    let rect_height_px = rect.height() * display.tile_size;
    for text in text_flow.iter() {
        let line_count = text_height(text, rect, metrics);
        let text_height_px = line_count * display.text_size;
        if ypos_px >= rect_height_px {
            return DrawResult::Overflow;
        }
        match text {
            Empty => {}

            EmptySpace(_) => {}

            Paragraph(text) => {
                let pos = rect.top_left() * display.tile_size + Point::new(0, ypos_px);
                let height = if ypos_px + text_height_px <= rect_height_px {
                    text_height_px
                } else {
                    rect_height_px - ypos_px
                };
                let options = TextOptions {
                    wrap: true,
                    width: rect.width(),
                    height: height,
                    skip,
                    ..Default::default()
                };
                let res =
                    display.draw_text_in_pixel_coordinates(pos, text, palette.gui_text, options);
                if let DrawResult::Overflow = res {
                    return res;
                };
            }

            Centered(text) => {
                let pos = rect.top_left() * display.tile_size + Point::new(0, ypos_px);
                let options = TextOptions {
                    skip,
                    ..TextOptions::align_center(rect.width())
                };
                let res =
                    display.draw_text_in_pixel_coordinates(pos, text, palette.gui_text, options);
                if let DrawResult::Overflow = res {
                    return res;
                };
            }

            // NOTE: this is no longer doing anything special! Maybe remove it later on?
            // Or handle this in engine/text renderer when we produce the characters.
            // Like, have an option that would always set the advance-width
            // to the tile width.
            SquareTiles(text) => {
                let pos = rect.top_left() * display.tile_size + Point::new(0, ypos_px);
                let options = TextOptions {
                    skip,
                    ..TextOptions::align_center(rect.width())
                };
                display.draw_text_in_pixel_coordinates(pos, text, palette.gui_text, options);
            }
        }
        ypos_px += text_height_px;

        if text_height_px < skip * display.text_size {
            ypos_px -= text_height_px;
            skip -= line_count;
        } else {
            ypos_px -= skip * display.text_size;
            skip = 0;
        }
    }

    DrawResult::Fit
}

fn text_height(text: &Text<'_>, rect: Rectangle, metrics: &dyn TextMetrics) -> i32 {
    use self::Text::*;
    match text {
        Empty => 1,
        EmptySpace(number_of_lines) => *number_of_lines,
        Paragraph(text) => {
            let options = TextOptions {
                wrap: true,
                width: rect.width(),
                ..Default::default()
            };
            metrics.get_text_height(text, options)
        }
        Centered(_text) => 1,
        SquareTiles(_text) => 1,
    }
}

pub fn text_flow_rect(
    text_flow: &[Text<'_>],
    rect: Rectangle,
    metrics: &dyn TextMetrics,
) -> Rectangle {
    let height = text_flow
        .iter()
        .map(|text| text_height(text, rect, metrics))
        .sum();
    Rectangle::new(rect.top_left(), rect.top_left() + (0, height))
}

pub fn text_rect(text: &Text<'_>, rect: Rectangle, metrics: &dyn TextMetrics) -> Rectangle {
    let height = text_height(text, rect, metrics);
    Rectangle::new(
        rect.top_left(),
        Point::new(rect.bottom_right().x, rect.top_left().y + height - 1),
    )
}

/// Helper for creating an egui button with the default background and
/// an enabled state.
pub fn button(text: &str, enabled: bool, palette: &Palette) -> egui::Button {
    let color = match enabled {
        true => palette.gui_text,
        false => palette.gui_text_inactive,
    };
    egui::Button::new(text)
        .fill(Some(palette.gui_button_background.into()))
        .text_color(color.into())
        .enabled(enabled)
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
    image_color: Color32,
    text_color: Color32,
    text_disabled_color: Color32,
    background_color: Color32,
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
            image_color: color::WHITE.into(),
            text_color: color::WHITE.into(),
            text_disabled_color: color::WHITE.into(),
            background_color: color::BLACK.into(),
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

    pub fn background_color(mut self, color: impl Into<Color32>) -> Self {
        self.background_color = color.into();
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
            image_color,
            text_color,
            text_disabled_color,
            background_color,
        } = self;

        let text_style = egui::TextStyle::Button;
        let font = &ui.fonts()[text_style];
        let prefix_galley = font.layout_no_wrap(prefix_text);
        let text_galley = font.layout_no_wrap(text);

        let uv = {
            let (x, y, tw, th, tilesize) = match self.texture {
                Texture::Tilemap => {
                    // TODO: Get the tilemap_size properly rather than hardcoding.
                    let tilemap_size = 180.0;
                    // NOTE: the graphical texture only has the 10px size. Still, don't hardcode
                    let tilesize = 10.0;
                    let (x, y) = graphic::tilemap_coords_px(0, graphic).unwrap_or((0, 0));
                    (x, y, tilemap_size, tilemap_size, tilesize)
                }
                Texture::Glyph => {
                    let tilesize = text_galley.size.y;
                    let tilemap_width = crate::engine::TILE_TEXTURE_WIDTH as f32;
                    let tilemap_height = crate::engine::TILE_TEXTURE_HEIGHT as f32;
                    let (x, y) =
                        crate::engine::glyph_coords_px_from_char(tilesize as u32, graphic.into())
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

            egui::Rect::from_min_size(
                (x as f32 / tw, y as f32 / th).into(),
                Vec2::new(tilesize / tw, tilesize / th),
            )
        };

        let image = widgets::Image::new(texture.into(), Vec2::splat(text_galley.size.y))
            .uv(uv)
            .tint(image_color)
            .bg_fill(background_color);

        let button_padding = ui.spacing().button_padding;
        let size = Vec2::new(prefix_galley.size.x + button_padding.x, button_padding.y)
            + (image.size() + 3.0 * button_padding)
            + Vec2::new(text_galley.size.x, 0.0);
        let (rect, response) = ui.allocate_exact_size(size, sense);
        let text_pos = rect.min
            + Vec2::new(prefix_galley.size.x, 0.0)
            + Vec2::new(image.size().x + button_padding.x * 2.0, button_padding.y);
        let prefix_translate = Vec2::new(prefix_galley.size.x + 2.0, button_padding.y);

        if ui.clip_rect().intersects(rect) {
            let visuals = ui.style().interact(&response);

            let painter = ui.painter();

            if selected {
                painter.rect(
                    rect,
                    visuals.corner_radius,
                    background_color,
                    visuals.bg_stroke,
                );
                painter.galley(
                    rect.min + button_padding,
                    prefix_galley,
                    text_style,
                    text_color,
                );
                painter.galley(text_pos, text_galley, text_style, text_color);
            } else if frame {
                painter.rect(
                    rect.expand(visuals.expansion),
                    visuals.corner_radius,
                    background_color,
                    visuals.bg_stroke,
                );
                painter.galley(
                    rect.min + button_padding,
                    prefix_galley,
                    text_style,
                    text_disabled_color,
                );
                painter.galley(text_pos, text_galley, text_style, text_disabled_color);
            }

            let image_rect = ui.layout().align_size_within_rect(
                image.size(),
                rect.shrink2(button_padding).translate(prefix_translate),
            );
            image.paint_at(ui, image_rect);
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
    use egui::paint::{Shape, Stroke};

    let percent = crate::util::clampf(0.0, percent, 1.0);

    let background_rect = Shape::Rect {
        rect: Rect::from_min_size(top_left, [width, height].into()),
        corner_radius: 0.0,
        stroke: Stroke::none(),
        fill: bg_color.into(),
    };
    let foreground_rect = Shape::Rect {
        rect: Rect::from_min_size(top_left, [width * percent, height].into()),
        corner_radius: 0.0,
        stroke: Stroke::none(),
        fill: fg_color.into(),
    };

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
