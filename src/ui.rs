#![allow(dead_code)]

use crate::{
    color::{self, Color},
    engine::{Display, DrawResult, TextMetrics, TextOptions},
    point::Point,
    rect::Rectangle,
};

use egui::{self, Ui};

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
                    display.draw_text_in_pixel_coordinates(pos, text, color::gui_text, options);
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
                    display.draw_text_in_pixel_coordinates(pos, text, color::gui_text, options);
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
                display.draw_text_in_pixel_coordinates(pos, text, color::gui_text, options);
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
pub fn button(text: &str, enabled: bool) -> egui::Button {
    egui::Button::new(text)
        .fill(Some(color::gui_button_background.into()))
        .enabled(enabled)
}

pub fn progress_bar(
    ui: &mut Ui,
    bg_cmd_index: egui::PaintCmdIdx,
    fg_cmd_index: egui::PaintCmdIdx,
    top_left: egui::Pos2,
    width: f32,
    height: f32,
    percent: f32,
    bg_color: Color,
    fg_color: Color,
) {
    use egui::{paint::PaintCmd, Rect};

    let percent = crate::util::clampf(0.0, percent, 1.0);

    let background_rect = PaintCmd::Rect {
        rect: Rect::from_min_size(top_left, [width, height].into()),
        corner_radius: 0.0,
        outline: None,
        fill: Some(bg_color.into()),
    };
    let foreground_rect = PaintCmd::Rect {
        rect: Rect::from_min_size(top_left, [width * percent, height].into()),
        corner_radius: 0.0,
        outline: None,
        fill: Some(fg_color.into()),
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
    pub fn new(pos: Point, text: &str) -> Self {
        Button {
            pos,
            text: text.into(),
            color: color::gui_text,
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
