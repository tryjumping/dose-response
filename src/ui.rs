use crate::{
    color::{self, Color},
    engine::{Display, TextMetrics, TextOptions},
    point::Point,
    rect::Rectangle,
};

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
    metrics: &dyn TextMetrics,
    display: &mut Display,
) {
    use self::Text::*;

    let mut ypos = 0;
    for text in text_flow.iter() {
        match text {
            Empty => {}

            EmptySpace(_) => {}

            Paragraph(text) => {
                let pos = rect.top_left() + Point::new(0, ypos);
                let options = TextOptions {
                    wrap: true,
                    width: rect.width(),
                    ..Default::default()
                };
                display.draw_text(pos, text, color::gui_text, options);
            }

            Centered(text) => {
                let pos = rect.top_left() + Point::new(0, ypos);
                display.draw_text(
                    pos,
                    text,
                    color::gui_text,
                    TextOptions::align_center(rect.width()),
                );
            }

            // NOTE: this is no longer doing anything special! Maybe remove it later on?
            // Or handle this in engine/text renderer when we produce the characters.
            // Like, have an option that would always set the advance-width
            // to the tile width.
            SquareTiles(text) => {
                let pos = rect.top_left() + Point::new(0, ypos);
                display.draw_text(
                    pos,
                    text,
                    color::gui_text,
                    TextOptions::align_center(rect.width()),
                );
            }
        }
        ypos += text_height(text, rect, metrics);
    }
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
