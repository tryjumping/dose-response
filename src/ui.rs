use color;
use engine::{Draw, TextMetrics, TextOptions};
use point::Point;
use rect::Rectangle;

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Text<'a> {
    Centered(&'a str),
    Empty,
    EmptySpace(i32),
    Paragraph(&'a str),
    SquareTiles(&'a str),
}

pub fn render_text_flow(
    text_flow: &[Text],
    rect: Rectangle,
    metrics: &TextMetrics,
    drawcalls: &mut Vec<Draw>,
) {
    use self::Text::*;

    let mut ypos = 0;
    for text in text_flow.iter() {
        match text {
            &Empty => {}

            &EmptySpace(_) => {}

            &Paragraph(text) => {
                let pos = rect.top_left() + Point::new(0, ypos);
                let options = TextOptions {
                    wrap: true,
                    width: rect.width(),
                    ..Default::default()
                };
                let dc = Draw::Text(pos, text.to_string().into(), color::gui_text, options);
                drawcalls.push(dc);
            }

            &Centered(text) => {
                let pos = rect.top_left() + Point::new(0, ypos);
                let dc = Draw::Text(
                    pos,
                    text.to_string().into(),
                    color::gui_text,
                    TextOptions::align_center(rect.width()),
                );
                drawcalls.push(dc);
            }

            // NOTE: this is no longer doing anything special! Maybe remove it later on?
            // Or handle this in engine/text renderer when we produce the characters.
            // Like, have an option that would always set the advance-width 
            // to the tile width.
            &SquareTiles(text) => {
                let pos = rect.top_left() + Point::new(0, ypos);
                let dc = Draw::Text(
                    pos,
                    text.to_string().into(),
                    color::gui_text,
                    TextOptions::align_center(rect.width()),
                );
                drawcalls.push(dc);
            }
        }
        ypos += text_height(text, rect, metrics);
    }
}

fn text_height(text: &Text, rect: Rectangle, metrics: &TextMetrics) -> i32 {
    use self::Text::*;
    match text {
        &Empty => 1,
        &EmptySpace(number_of_lines) => number_of_lines,
        &Paragraph(text) => {
            let pos = rect.top_left();
            let options = TextOptions {
                wrap: true,
                width: rect.width(),
                ..Default::default()
            };
            let dc = Draw::Text(pos, text.to_string().into(), color::gui_text, options);
            metrics.get_text_height(&dc)
        }
        &Centered(_text) => 1,
        &SquareTiles(_text) => 1,
    }
}

pub fn text_flow_rect(text_flow: &[Text], rect: Rectangle, metrics: &TextMetrics) -> Rectangle {
    let height = text_flow
        .iter()
        .map(|text| text_height(text, rect, metrics))
        .sum();
    Rectangle::new(rect.top_left(), rect.top_left() + (0, height))
}

pub fn text_rect(text: &Text, rect: Rectangle, metrics: &TextMetrics) -> Rectangle {
    let height = text_height(text, rect, metrics);
    Rectangle::new(
        rect.top_left(),
        Point::new(rect.bottom_right().x, rect.top_left().y + height - 1),
    )
}
