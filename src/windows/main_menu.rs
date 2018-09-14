use color;
use engine::{Display, TextMetrics};
use point::Point;
use rect::Rectangle;
use state::State;
use ui::{self, Text};

#[derive(Debug)]
pub enum MenuItem {
    Resume,
    NewGame,
    Help,
    ToggleFullscreen,
    SaveAndQuit,
    Load,
    Quit,
}

impl MenuItem {
    pub fn to_str(&self) -> &'static str {
        use self::MenuItem::*;
        match self {
            Resume => "[R]esume",
            NewGame => "[N]ew Game",
            Help => "[H]elp",
            ToggleFullscreen => "[F]ullscreen",
            SaveAndQuit => "[S]ave and Quit",
            Load => "[L]oad game",
            // TODO:
            // "[Q]uit without saving"
            Quit => "[Q]uit",
        }
    }
}

pub struct Layout<'a> {
    window_rect: Rectangle,
    inner_window_rect: Rectangle,
    rect: Rectangle,
    text_flow: Vec<Text<'a>>,
    menu_item_under_mouse: Option<MenuItem>,
    menu_rect_under_mouse: Option<Rectangle>,
}

pub struct Window;

impl Window {
    fn calculate_layout(&self, state: &State, metrics: &TextMetrics) -> Layout {
        use ui::Text::*;

        let window_pos = Point::new(0, 0);
        let window_size = state.display_size;

        let window_rect = Rectangle::from_point_and_size(window_pos, window_size);

        let inner_window_rect = Rectangle::new(
            window_rect.top_left() + (1, 1),
            window_rect.bottom_right() - (1, 1),
        );

        let rect_size = Point::new(20, 15);
        let rect_pos = Point::new((inner_window_rect.width() - rect_size.x) / 2, 0);
        let rect = Rectangle::from_point_and_size(rect_pos, rect_size);

        let mut text_flow = vec![
            EmptySpace(2),
            Centered("Dose Response"),
            Centered("By Tomas Sedovic"),
            EmptySpace(2),
        ];

        let mut options = vec![];

        if !state.game_ended {
            options.push(MenuItem::Resume);
        }

        options.push(MenuItem::NewGame);

        // NOTE: we won't hiding this option, because it would require
        // checking if the file exists every frame (or do some complex
        // caching).
        options.push(MenuItem::Load);

        options.push(MenuItem::Help);

        if cfg!(feature = "fullscreen") {
            options.push(MenuItem::ToggleFullscreen);
        }

        if state.game_ended {
            options.push(MenuItem::Quit);
        } else {
            options.push(MenuItem::SaveAndQuit);
        }

        let header_rect = ui::text_flow_rect(&text_flow, rect, metrics);

        let mut menu_item_under_mouse = None;
        let mut menu_rect_under_mouse = None;
        let mut ypos = header_rect.bottom_right().y;
        for option in options {
            let text = Centered(option.to_str());
            let text_rect = ui::text_rect(
                &text,
                Rectangle::new(rect.top_left() + (0, ypos), rect.bottom_right()),
                metrics,
            );
            ypos += text_rect.size().y;
            if text_rect.contains(state.mouse.tile_pos) {
                menu_item_under_mouse = Some(option);
                menu_rect_under_mouse = Some(text_rect);
            }
            text_flow.push(text);
            text_flow.push(Empty);
            ypos += ui::text_rect(
                &Empty,
                Rectangle::new(rect.top_left() + (0, ypos), rect.bottom_right()),
                metrics,
            ).size()
            .y;
        }

        text_flow.push(EmptySpace(3));
        text_flow.push(Paragraph("\"You cannot lose if you do not play.\""));
        text_flow.push(Paragraph("-- Marla Daniels"));

        Layout {
            window_rect,
            inner_window_rect,
            rect,
            text_flow,
            menu_item_under_mouse,
            menu_rect_under_mouse,
        }
    }

    pub fn render(&self, state: &State, metrics: &TextMetrics, display: &mut Display) {
        let layout = self.calculate_layout(state, metrics);
        display.draw_rectangle(layout.window_rect, color::window_edge);
        display.draw_rectangle(layout.inner_window_rect, color::background);

        let rect = layout.rect;

        if let Some(rect) = layout.menu_rect_under_mouse {
            display.draw_rectangle(rect, color::menu_highlight);
        }

        ui::render_text_flow(&layout.text_flow, rect, metrics, display);
    }

    pub fn hovered(&self, state: &State, metrics: &TextMetrics) -> Option<MenuItem> {
        self.calculate_layout(state, metrics).menu_item_under_mouse
    }
}
