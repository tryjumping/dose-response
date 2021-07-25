#![allow(unused_variables, dead_code)]
use crate::{
    engine::{Display, TextMetrics},
    point::Point,
    rect::Rectangle,
    state::State,
};

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

        display.draw_rectangle(layout.window_rect, state.palette.gui_window_background);

        // TODO: move this to egui
    }
}
