use crate::{
    color,
    engine::{Display, TextMetrics},
    point::Point,
    rect::Rectangle,
    state::State,
    ui,
};

pub enum OptionItem {
    Fullscreen,
    Window,
}

struct Layout {
    window_rect: Rectangle,
    rect: Rectangle,
}

pub struct Window;

impl Window {
    fn layout(&self, state: &State, _metrics: &dyn TextMetrics) -> Layout {
        let screen_padding = Point::from_i32(2);
        let window_rect = Rectangle::from_point_and_size(
            screen_padding,
            state.display_size - (screen_padding * 2),
        );

        let rect = Rectangle::new(
            window_rect.top_left() + (2, 0),
            window_rect.bottom_right() - (2, 1),
        );

        Layout { window_rect, rect }
    }

    pub fn render(&self, state: &State, metrics: &dyn TextMetrics, display: &mut Display) {
        use crate::ui::Text::*;

        let layout = self.layout(state, metrics);

        let lines = vec![
            Centered("Options"),
            Empty,
            Centered("[F]ullscreen / [W]indow"),
        ];

        display.draw_rectangle(layout.window_rect, color::window_edge);

        display.draw_rectangle(
            Rectangle::new(
                layout.window_rect.top_left() + (1, 1),
                layout.window_rect.bottom_right() - (1, 1),
            ),
            color::window_background,
        );

        ui::render_text_flow(&lines, layout.rect, metrics, display);
    }

    pub fn hovered(&self, _state: &State, _fwmetrics: &dyn TextMetrics) -> Option<OptionItem> {
        None
    }
}
