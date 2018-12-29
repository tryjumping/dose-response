use crate::{
    color,
    engine::{Display, Settings, TextMetrics},
    point::Point,
    rect::Rectangle,
    state::State,
    ui::{self, Button},
};

pub enum OptionItem {
    Fullscreen,
    Window,
}

struct Layout {
    window_rect: Rectangle,
    rect: Rectangle,
    option_under_mouse: Option<OptionItem>,
    rect_under_mouse: Option<Rectangle>,
    fullscreen_button: Button,
}

pub struct Window;

impl Window {
    fn layout(&self, state: &State, metrics: &dyn TextMetrics) -> Layout {
        let screen_padding = Point::from_i32(2);
        let window_rect = Rectangle::from_point_and_size(
            screen_padding,
            state.display_size - (screen_padding * 2),
        );

        let rect = Rectangle::new(
            window_rect.top_left() + (2, 0),
            window_rect.bottom_right() - (2, 1),
        );

        let mut option_under_mouse = None;
        let mut rect_under_mouse = None;

        let fullscreen_button = Button::new(rect.top_left(), "LOOOL").align_center(rect.width());

        let button_rect = metrics.button_rect(&fullscreen_button);
        if button_rect.contains(state.mouse.tile_pos) {
            option_under_mouse = Some(OptionItem::Fullscreen);
            rect_under_mouse = Some(button_rect);
        }

        Layout {
            window_rect,
            rect,
            option_under_mouse,
            rect_under_mouse,
            fullscreen_button,
        }
    }

    pub fn render(
        &self,
        state: &State,
        settings: &Settings,
        metrics: &dyn TextMetrics,
        display: &mut Display,
    ) {
        use crate::ui::Text::*;

        let layout = self.layout(state, metrics);

        let font_size = format!("Font size (current: {}):", settings.font_size);
        let sizes_str = crate::engine::AVAILABLE_FONT_SIZES
            .iter()
            .map(|num| num.to_string())
            .collect::<Vec<_>>();
        let sizes = sizes_str.join(" / ");

        let lines = vec![
            Centered("Settings"),
            Empty,
            Centered("Display:"),
            Centered("[F]ullscreen / [W]indow"),
            Empty,
            Centered(&font_size),
            Centered(&sizes),
            Empty,
            Centered("Graphics backend:"),
            Centered("Glutin / SDL"),
            Empty,
            Centered("Back"),
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

        if let Some(rect) = layout.rect_under_mouse {
            display.draw_rectangle(rect, color::menu_highlight);
        }

        //display.draw_button(&layout.fullscreen_button);
        let _ = layout.fullscreen_button;
    }

    pub fn hovered(&self, state: &State, metrics: &dyn TextMetrics) -> Option<OptionItem> {
        self.layout(state, metrics).option_under_mouse
    }
}
