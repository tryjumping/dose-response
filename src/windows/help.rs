use color;
use engine::{Draw, TextMetrics, TextOptions};
use point::Point;
use rect::Rectangle;
use state::State;
use ui;


pub enum Action {
    NextPage,
    PrevPage,
}


#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum Page {
    NumpadControls,
    ArrowControls,
    ViKeys,
    HowToPlay,
}


struct Layout {
    next_page_button: Option<Draw>,
    prev_page_button: Option<Draw>,
    action_under_mouse: Option<Action>,
    rect_under_mouse: Option<Rectangle>,
    window_rect: Rectangle,
    rect: Rectangle,
}


pub struct Window;


impl Window {
    fn layout(&self, state: &State, metrics: &TextMetrics) -> Layout {
        let screen_padding = Point::from_i32(2);
        let window_rect =
            Rectangle::from_point_and_size(screen_padding, state.display_size - (screen_padding * 2));

        let rect = Rectangle::new(
            window_rect.top_left() + (2, 1),
            window_rect.bottom_right() - (2, 1),
        );

        let mut action_under_mouse = None;
        let mut rect_under_mouse = None;

        let next_page_button = if state.current_help_window != Page::HowToPlay {
            let text = "[->] Next page";
            let drawcall = Draw::Text(
                rect.bottom_right(),
                text.into(),
                color::gui_text,
                TextOptions::align_right(),
            );
            let text_rect = metrics.text_rect(&drawcall);
            if text_rect.contains(state.mouse.tile_pos) {
                action_under_mouse = Some(Action::NextPage);
                rect_under_mouse = Some(text_rect);
            }
            Some(drawcall)
        } else {
            None
        };

        let prev_page_button = if state.current_help_window != Page::NumpadControls {
            let text = "Previous page [<-]";
            let drawcall = Draw::Text(
                rect.bottom_left(),
                text.into(),
                color::gui_text,
                Default::default(),
            );
            let text_rect = metrics.text_rect(&drawcall);
            if text_rect.contains(state.mouse.tile_pos) {
                action_under_mouse = Some(Action::PrevPage);
                rect_under_mouse = Some(text_rect);
            }
            Some(drawcall)
        } else {
            None
        };

        Layout {
            window_rect,
            rect,
            next_page_button,
            prev_page_button,
            action_under_mouse,
            rect_under_mouse,
        }
    }

    pub fn render(&self, state: &State, metrics: &TextMetrics, drawcalls: &mut Vec<Draw>) {
        use ui::Text::*;


        let layout = self.layout(state, metrics);

        drawcalls.push(Draw::Rectangle(layout.window_rect, color::window_edge,));

        drawcalls.push(Draw::Rectangle(
            Rectangle::new(layout.window_rect.top_left() + (1, 1),
                           layout.window_rect.bottom_right() - (1, 1)),
            color::background,
        ));

        let mut lines = vec![];

        match state.current_help_window {
            Page::NumpadControls => {
                lines.push(Centered("Controls: numpad"));
                lines.push(EmptySpace(2));

                lines.push(Paragraph("You control the @ character. It moves just like the king in Chess: one step in any direction. That means up, down, left, right, but also diagonally."));
                lines.push(Empty);
                lines.push(Paragraph("You can use the numpad. Imagine your @ is in the middle (where [5] is) and you just pick a direction."));
                lines.push(EmptySpace(3));

                lines.push(SquareTiles(r"7 8 9"));
                lines.push(SquareTiles(r" \|/ "));
                lines.push(SquareTiles(r"4-@-6"));
                lines.push(SquareTiles(r" /|\ "));
                lines.push(SquareTiles(r"1 2 3"));
            }

            Page::ArrowControls => {
                lines.push(Centered("Controls: arrow keys"));
                lines.push(EmptySpace(2));

                lines.push(Paragraph("You control the @ character. It moves just like the king in Chess: one step in any direction. That means up, down, left, right, but also diagonally."));
                lines.push(Empty);
                lines.push(Paragraph("If you don't have a numpad, you can use the arrow keys. You will need [Shift] and [Ctrl] for diagonal movement. [Shift] means up and [Ctrl] means down. You combine them with the [Left] and [Right] keys."));

                lines.push(EmptySpace(3));

                lines.push(SquareTiles(r"Shift+Left  Up  Shift+Right"));
                lines.push(SquareTiles(r"         \  |  /           "));
                lines.push(SquareTiles(r"       Left-@-Right        "));
                lines.push(SquareTiles(r"         /  |  \           "));
                lines.push(SquareTiles(r"Ctrl+Left  Down Ctrl+Right "));
            }

            Page::ViKeys => {
                lines.push(Centered("Controls: Vi keys"));
                lines.push(EmptySpace(2));

                lines.push(Paragraph("You control the @ character. It moves just like the king in Chess: one step in any direction. That means up, down, left, right, but also diagonally."));
                lines.push(Empty);
                lines.push(Paragraph("You can also move using the \"Vi keys\". Those map to the letters on your keyboard. This makes more sense if you've ever used the Vi text editor."));
                lines.push(EmptySpace(3));

                lines.push(SquareTiles(r"y k u"));
                lines.push(SquareTiles(r" \|/ "));
                lines.push(SquareTiles(r"h-@-l"));
                lines.push(SquareTiles(r" /|\ "));
                lines.push(SquareTiles(r"b j n"));
            }

            Page::HowToPlay => {
                lines.push(Centered("How to play"));
                lines.push(EmptySpace(2));

                lines.push(Paragraph("Your character ('@') is an addict. If you stay long without using a Dose ('i'), you will lose. You can also pick up food ('%') which lets you stay sober for longer."));
                lines.push(Empty);
                lines.push(Paragraph(
                    "Using a Dose or eating Food will also kill all nearby enemies.",
                ));
                lines.push(Empty);
                lines.push(Paragraph("Each Dose has a glow around it. If you step into it, you will be unable to resist even if it means Overdosing yourself. At the beginning, you will also Overdose by using another Dose when you're still High or using a Dose that's too strong for you ('+', 'x' or 'I'). With each Dose you build up tolerance which makes you seek out stronger Doses later on."));
                lines.push(Empty);
                lines.push(Paragraph("All the letters ('h', 'v', 'S', 'a' and 'D') are enemies. Each has their own way of harming you. The 'D' move twice as fast and will kill you outright. The 'a' will reduce your Will on each hit. When it reaches zero, you will lose."));
                lines.push(Empty);
                lines.push(Paragraph("To progress, you need to get stronger Will. Defeat enough `a` monsters and it will go up. The Dose or Food \"explosions\" don't count though! Higher Will makes the irresistible area around Doses smaller. It will also let you pick them up!"));
                lines.push(Empty);
                lines.push(Paragraph("If you see another @ characters, they are friendly. They will give you a bonus and follow you around, but only while you're Sober."));
            }
        }

        ui::render_text_flow(&lines, layout.rect, metrics, drawcalls);

        if let Some(highlighted_rect) = layout.rect_under_mouse {
            drawcalls.push(
                Draw::Rectangle(highlighted_rect,
                                color::menu_highlight));
        }

        if let Some(drawcall) = layout.next_page_button {
            drawcalls.push(drawcall)
        }

        if let Some(drawcall) = layout.prev_page_button {
            drawcalls.push(drawcall)
        }
    }


    pub fn hovered(&self, state: &State, metrics: &TextMetrics) -> Option<Action> {
        self.layout(state, metrics).action_under_mouse
    }
}
