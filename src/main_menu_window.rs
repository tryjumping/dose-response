use color;
use engine::{Draw, Mouse};
use point::Point;
use rect::Rectangle;
use state::State;
use ui::{self, Layout};


pub enum Options {
    NewGame,
    Help,
    SaveAndQuit,
    Load,
    Quit,
}


pub struct Window;


impl Window {
    fn calculate_layout(&self, state: &State) -> Layout {
        use ui::Layout::*;

        let window_pos = Point::new(0, 0);
        let window_size = state.display_size;

        let window_rect = Rectangle::from_point_and_size(window_pos, window_size);

        let inner_window_rect = Rectangle::from_point_and_size(
            window_rect.top_left() + (1, 1), window_rect.dimensions() - (2, 2));

        let rect_size = Point::new(20, 15);
        let rect_pos = Point::new((inner_window_rect.width() - rect_size.x) / 2, 0);
        let rect = Rectangle::from_point_and_size(rect_pos, rect_size);

        let mut lines = vec![
            EmptySpace(2),
            Centered("Dose Response"),
            Centered("By Tomas Sedovic"),
            EmptySpace(2),
        ];

        let next_y = ui::render_laid_out_text(&lines, rect, metrics, drawcalls);

        let mut options = vec![];

        if !state.game_ended {
            options.push("[R]esume");
        }

        options.push("[N]ew Game");

        // NOTE: we won't hiding this option, because it would require
        // checking if the file exists every frame (or do some complex
        // caching).
        options.push("[L]oad game");

        options.push("[H]elp");

        if !state.game_ended {
            options.push("[S]ave and Quit");
        }

        if state.game_ended {
            options.push("[Q]uit");
        } else {
            options.push("[Q]uit without saving");
        }


        for option in options {
            let rect = layout_text_rect(Centered(option));
            if rect.contains(mouse.pos) {
                return Some(option)
            }
            lines.push(Centered(option));
            lines.push(Empty);
        }
        let rect = Rectangle::new(rect.top_left() + (0, next_y), rect.bottom_right());
        let rect_under_mouse = rect_under_mouse(&lines, rect, metrics, state.mouse);


        lines.push(EmptySpace(3));
        lines.push(Paragraph("\"You cannot lose if you do not play.\""));
        lines.push(Paragraph("-- Marla Daniels"));

    }


    pub fn render(&self, state: &State, drawcalls: &mut Vec<Draw>) {

        let layout = window.calculate_layout();
        drawcalls.push(
            Draw::Rectangle(
                layout.window_rect.top_left(),
                layout.window_rect.dimensions(),
                color::window_edge));
        drawcalls.push(
            Draw::Rectangle(
                layout.inner_window_rect.top_left(),
                layout.inner_window_rect.dimensions(),
                color::background));

        let rect = layout.rect;

        render_laid_out_text(&layout.lines, rect, metrics, drawcalls);

        if let Some(rect) = layout.highlighted_option_rect {
            drawcalls.push(Draw::Rectangle(rect.top_left(), rect.dimensions(), color::red));
        }
    }

    pub fn hovered(mouse: Mouse) -> Option<Options> {
        window.calculate_layout().highlighted_option
    }
}
