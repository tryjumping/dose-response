#![allow(warnings)]
use crate::color;
use crate::engine::{Display, TextMetrics};
use crate::formula;
use crate::player::CauseOfDeath;
use crate::point::Point;
use crate::rect::Rectangle;
use crate::state::{Side, State};
use crate::ui::{self, Button};

struct Layout {
    window_rect: Rectangle,
    rect: Rectangle,
}

/// This window appears only during a gameplay recording/replay.
///
/// It is not interactive and should not be visible during normal
/// playtime.
pub struct Window;

impl Window {
    fn layout(&self, state: &State, metrics: &dyn TextMetrics, display: &mut Display) -> Layout {
        let padding = Point::from_i32(1);
        let size = Point::new(37, 17) + (padding * 2);
        let top_left = Point {
            x: (display.size_without_padding().x - size.x) / 2,
            y: 7,
        };

        let window_rect = Rectangle::from_point_and_size(top_left, size);

        let rect = Rectangle::new(
            window_rect.top_left() + padding,
            window_rect.bottom_right() - padding,
        );

        Layout { window_rect, rect }
    }

    pub fn render(&self, state: &State, metrics: &dyn TextMetrics, display: &mut Display) {
        use crate::ui::Text::*;

        let layout = self.layout(state, metrics, display);

        let lines = vec![
            EmptySpace(1),
            Centered("Dose Response"),
            Centered("By Tomas Sedovic"),
            EmptySpace(2),
            Centered("Visit:"),
            Centered("https://tryjumping.com"),
            EmptySpace(6),
            Centered("\"You cannot lose if you do not play.\""),
            Centered("-- Marla Daniels"),
        ];

        display.draw_rectangle(layout.window_rect, color::window_background);

        ui::render_text_flow(&lines, layout.rect, 0, metrics, display);
    }
}
