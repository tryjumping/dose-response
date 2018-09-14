use color::Color;
use engine::Display;
use point::Point;
use rect::Rectangle;

pub fn progress_bar(
    display: &mut Display,
    percentage: f32,
    pos: Point,
    width: i32,
    foreground: Color,
    background: Color,
) {
    assert!(percentage >= 0.0);
    assert!(percentage <= 1.0);
    let mut highlighted_width = (width as f32 * percentage) as i32;
    if percentage > 0.0 && highlighted_width == 0 {
        highlighted_width = 1;
    } else if percentage >= 1.0 {
        highlighted_width = width;
    }

    if highlighted_width > 0 {
        display.draw_rectangle(
            Rectangle::from_point_and_size(pos, Point::new(highlighted_width, 1)),
            foreground,
        );
    }

    let remaining_width = width - highlighted_width;
    if remaining_width > 0 {
        display.draw_rectangle(
            Rectangle::from_point_and_size(
                pos + (highlighted_width, 0),
                Point::new(remaining_width, 1),
            ),
            background,
        );
    }
}

pub fn lerp_f32(from: f32, to: f32, t: f32) -> f32 {
    (1.0 - t) * from + t * to
}

pub fn lerp_u8(from: u8, to: u8, t: f32) -> u8 {
    lerp_f32(f32::from(from), f32::from(to), t) as u8
}

pub fn fade_color(from: Color, to: Color, progress: f32) -> Color {
    debug_assert!(progress >= 0.0);
    debug_assert!(progress <= 1.0);
    Color {
        r: lerp_u8(from.r, to.r, progress),
        g: lerp_u8(from.g, to.g, progress),
        b: lerp_u8(from.b, to.b, progress),
    }
}
