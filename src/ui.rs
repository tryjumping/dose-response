use color;
use engine::{Draw, TextMetrics, TextOptions};
use point::Point;
use rect::Rectangle;

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum TextFlow<'a> {
    Centered(&'a str),
    Empty,
    EmptySpace(i32),
    Paragraph(&'a str),
    SquareTiles(&'a str),
}

pub fn render_text_flow(
    lines: &[TextFlow],
    rect: Rectangle,
    metrics: &TextMetrics,
    drawcalls: &mut Vec<Draw>,
) {
    use self::TextFlow::*;

    let mut ypos = 0;
    for text in lines.iter() {
        match text {
            &Empty => {
                ypos += 1;
            }

            &EmptySpace(number_of_lines) => {
                ypos += number_of_lines;
            }

            &Paragraph(text) => {
                let pos = rect.top_left() + Point::new(0, ypos);
                let options = TextOptions {
                    wrap: true,
                    width: rect.width(),
                    ..Default::default()
                };
                let dc = Draw::Text(pos, text.to_string().into(), color::gui_text, options);
                ypos += metrics.get_text_height(&dc);
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
                ypos += 1;
                drawcalls.push(dc);
            }

            &SquareTiles(text) => {
                let text_size = text.chars().count() as i32;
                let max_size = rect.width();
                let start_pos = rect.top_left() + ((max_size - text_size) / 2, ypos);
                for (i, chr) in text.char_indices() {
                    let pos = start_pos + (i as i32, 0);
                    drawcalls.push(Draw::Char(pos, chr, color::gui_text));
                }
                ypos += 1;
            }
        }
    }
}

pub fn text_flow_rect(
    lines: &[TextFlow],
    rect: Rectangle,
    metrics: &TextMetrics,
) -> Rectangle {
    //use self::TextFlow::*;
    unimplemented!()
}

pub fn text_rect(
    text_flow: &TextFlow,
    rect: Rectangle,
    metrics: &TextMetrics,
) -> Rectangle {
    //use self::TextFlow::*;
    unimplemented!()
}
