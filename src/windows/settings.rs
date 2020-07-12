use crate::{
    color,
    engine::{Display, TextMetrics},
    point::Point,
    rect::Rectangle,
    settings::Settings,
    state::State,
};

use egui::{self, label, Align, Button, Ui, Window as GuiWindow};

pub enum Action {
    Fullscreen,
    Window,
    TileSize(i32),
    TextSize(i32),
    Back,
    Apply,
}

struct Layout {
    window_rect: Rectangle,
    rect: Rectangle,
    option_under_mouse: Option<Action>,
    rect_under_mouse: Option<Rectangle>,
    fullscreen_button: Button,
    window_button: Button,
    tile_size_options: Vec<(i32, Button)>,
    text_size_options: Vec<(i32, Button)>,
    back_button: Button,
    apply_button: Button,
}

pub struct Window;

impl Window {
    // fn layout(
    //     &self,
    //     state: &State,
    //     _settings: &Settings,
    //     metrics: &dyn TextMetrics,
    //     display: &Display,
    //     top_level: bool,
    // ) -> Layout {
    //     let screen_padding = Point::from_i32(2);
    //     let window_rect = Rectangle::from_point_and_size(
    //         screen_padding,
    //         display.size_without_padding() - (screen_padding * 2),
    //     );

    //     let rect = Rectangle::new(
    //         window_rect.top_left() + (2, 0),
    //         window_rect.bottom_right() - (2, 1),
    //     );

    //     let mut option_under_mouse = None;
    //     let mut rect_under_mouse = None;

    //     let midpoint = rect.top_left() + (rect.width() / 2, 3);
    //     let fullscreen_button = Button::new(midpoint - (1, 0), "[F]ullscreen").align_right();
    //     let window_button = Button::new(midpoint + (1, 0), "[W]indow").align_left();
    //     let apply_button =
    //         Button::new(rect.bottom_right() + (0, -1), "[A]pply settings").align_right();
    //     let back_button = Button::new(rect.bottom_left() + (0, -1), "[Esc] Back").align_left();

    //     let button_rect = metrics.button_rect(&fullscreen_button);
    //     if button_rect.contains(state.mouse.tile_pos) {
    //         option_under_mouse = Some(Action::Fullscreen);
    //         rect_under_mouse = Some(button_rect);
    //     }

    //     let button_rect = metrics.button_rect(&window_button);
    //     if button_rect.contains(state.mouse.tile_pos) {
    //         option_under_mouse = Some(Action::Window);
    //         rect_under_mouse = Some(button_rect);
    //     }

    //     let button_rect = metrics.button_rect(&back_button);
    //     if button_rect.contains(state.mouse.tile_pos) {
    //         option_under_mouse = Some(Action::Back);
    //         rect_under_mouse = Some(button_rect);
    //     }

    //     let button_rect = metrics.button_rect(&apply_button);
    //     if button_rect.contains(state.mouse.tile_pos) {
    //         option_under_mouse = Some(Action::Apply);
    //         rect_under_mouse = Some(button_rect);
    //     }

    //     let tile_size_texts = crate::engine::AVAILABLE_TILE_SIZES
    //         .iter()
    //         .rev()
    //         .enumerate()
    //         .map(|(index, &tile_size)| {
    //             let text = format!("[{}] {}px", index + 1, tile_size,);
    //             (tile_size, text)
    //         })
    //         .collect::<Vec<_>>();
    //     let text_size_option_width =
    //         metrics.get_text_width(&tile_size_texts[0].1, Default::default());

    //     let x_offset = (rect.width() - text_size_option_width) / 2;
    //     let tile_size_options = tile_size_texts
    //         .iter()
    //         .enumerate()
    //         .map(|(index, &(tile_size, ref text))| {
    //             let button = Button::new(rect.top_left() + (x_offset, 6 + index as i32), text);
    //             (tile_size, button)
    //         })
    //         .collect::<Vec<_>>();

    //     for (size, button) in &tile_size_options {
    //         let button_rect = metrics.button_rect(&button);
    //         if button_rect.contains(state.mouse.tile_pos) {
    //             option_under_mouse = Some(Action::TileSize(*size));
    //             rect_under_mouse = Some(button_rect);
    //         }
    //     }

    //     let text_size_texts = crate::engine::AVAILABLE_TEXT_SIZES
    //         .iter()
    //         .rev()
    //         .enumerate()
    //         .map(|(index, &tile_size)| {
    //             let text = format!(
    //                 "[{}] {}px",
    //                 crate::engine::AVAILABLE_TILE_SIZES.len() + index + 1,
    //                 tile_size
    //             );
    //             (tile_size, text)
    //         })
    //         .collect::<Vec<_>>();

    //     let text_size_options = text_size_texts
    //         .iter()
    //         .enumerate()
    //         .map(|(index, &(text_size, ref text))| {
    //             let y_offset = 6 + crate::engine::AVAILABLE_TILE_SIZES.len() as i32;
    //             let button =
    //                 Button::new(rect.top_left() + (x_offset, y_offset + index as i32), text);
    //             (text_size, button)
    //         })
    //         .collect::<Vec<_>>();

    //     for (size, button) in &text_size_options {
    //         let button_rect = metrics.button_rect(&button);
    //         if button_rect.contains(state.mouse.tile_pos) {
    //             option_under_mouse = Some(Action::TextSize(*size));
    //             rect_under_mouse = Some(button_rect);
    //         }
    //     }

    //     if !top_level {
    //         option_under_mouse = None;
    //         rect_under_mouse = None;
    //     }

    //     Layout {
    //         window_rect,
    //         rect,
    //         option_under_mouse,
    //         rect_under_mouse,
    //         fullscreen_button,
    //         window_button,
    //         tile_size_options,
    //         text_size_options,
    //         back_button,
    //         apply_button,
    //     }
    // }

    pub fn process(
        &self,
        state: &State,
        ui: &mut Ui,
        settings: &Settings,
        _metrics: &dyn TextMetrics,
        _display: &mut Display,
        _top_level: bool,
    ) -> Option<Action> {
        let mut visible = true;

        // NOTE: this is why I think it probably makes sense to keep
        // the logic and rendering in the same place. We won't have to
        // be returning actions or whatnot to process them later. But
        // IDK might lead to spagetti code and right now, the GUI
        // layout and the code is cleanly separate. IDK.
        let mut action = None;

        GuiWindow::new("Settings")
            .open(&mut visible)
            // TODO: make sure this fits the game window
            .fixed_size([800.0, 500.0])
            .show(ui.ctx(), |ui| {
                ui.columns(2, |c| {
                    c[0].label("Display:");
                    if c[0].radio("[F]ullscreen", settings.fullscreen).clicked {
                        action = Some(Action::Fullscreen);
                    }
                    if c[0].radio("[W]indowed", !settings.fullscreen).clicked {
                        action = Some(Action::Window)
                    }

                    let mut available_key_shortcut = 1;

                    c[0].label("Tile Size:");
                    for &tile_size in crate::engine::AVAILABLE_TILE_SIZES.iter().rev() {
                        let selected = tile_size == settings.tile_size;
                        if c[0]
                            .radio(
                                format!("[{}] {}px", available_key_shortcut, tile_size),
                                selected,
                            )
                            .clicked
                        {
                            action = Some(Action::TileSize(tile_size));
                        };
                        available_key_shortcut += 1;
                    }

                    c[0].label("Text Size:");
                    for &text_size in crate::engine::AVAILABLE_TEXT_SIZES.iter().rev() {
                        let selected = text_size == settings.text_size;
                        if c[0]
                            .radio(
                                format!("[{}] {}px", available_key_shortcut, text_size),
                                selected,
                            )
                            .clicked
                        {
                            action = Some(Action::TextSize(text_size));
                        };
                        available_key_shortcut += 1;
                    }

                    c[1].label("Accessibility:");
                    c[1].label("Tiles:");
                    c[1].radio("[G]raphical", true);
                    c[1].radio("[T]extual (ASCII)", false);

                    c[1].label("Colour:");
                    c[1].radio("[S]tandard", true);
                    c[1].radio("[C]olour-blind", false);
                    c[1].radio("C[u]stom", false);
                });

                ui.separator();
                ui.horizontal(|ui| {
                    if ui.button("[A]pply Changes").clicked {
                        action = Some(Action::Apply);
                    }

                    if ui.button("[B]ack").clicked {
                        action = Some(Action::Back);
                    }
                });
            });

        if !visible {
            return Some(Action::Back);
        }

        action
    }

    // pub fn render(
    //     &self,
    //     state: &State,
    //     settings: &Settings,
    //     metrics: &dyn TextMetrics,
    //     display: &mut Display,
    //     top_level: bool,
    // ) {
    //     use crate::ui::Text::*;

    //     let layout = self.layout(state, settings, metrics, display, top_level);

    //     display.draw_rectangle(layout.window_rect, color::window_edge);

    //     display.draw_rectangle(
    //         Rectangle::new(
    //             layout.window_rect.top_left() + (1, 1),
    //             layout.window_rect.bottom_right() - (1, 1),
    //         ),
    //         color::window_background,
    //     );

    //     let current_display_mode = if settings.fullscreen {
    //         "fullscreen"
    //     } else {
    //         "window"
    //     };
    //     let display_header = format!("Display (current: {}):", current_display_mode);
    //     let tile_size = format!("Tile size (current: {}px):", settings.tile_size);
    //     let text_size = format!("text size (current: {}px):", settings.text_size);

    //     let lines = vec![
    //         Centered("Settings"),
    //         Empty,
    //         Centered(&display_header),
    //         Centered(" "), // Fullscreen / Window
    //         Empty,
    //         Centered(&tile_size),
    //         EmptySpace(crate::engine::AVAILABLE_TILE_SIZES.len() as i32),
    //         Empty,
    //         Centered(&text_size),
    //         EmptySpace(crate::engine::AVAILABLE_TEXT_SIZES.len() as i32),
    //     ];

    //     ui::render_text_flow(&lines, layout.rect, 0, metrics, display);

    //     if let Some(rect) = layout.rect_under_mouse {
    //         display.draw_rectangle(rect, color::menu_highlight);
    //     }

    //     // Highlight the active Fullscreen or Window option
    //     {
    //         let rect = if settings.fullscreen {
    //             metrics.button_rect(&layout.fullscreen_button)
    //         } else {
    //             metrics.button_rect(&layout.window_button)
    //         };
    //         display.draw_rectangle(rect, color::dim_background);
    //     }

    //     display.draw_button(&layout.fullscreen_button);
    //     display.draw_button(&layout.window_button);

    //     for (size, button) in &layout.tile_size_options {
    //         // Highlight the active tile size
    //         if *size == settings.tile_size {
    //             let rect = metrics.button_rect(button);
    //             display.draw_rectangle(rect, color::dim_background);
    //         }
    //         display.draw_button(button)
    //     }

    //     for (size, button) in &layout.text_size_options {
    //         // Highlight the active text size
    //         if *size == settings.text_size {
    //             let rect = metrics.button_rect(button);
    //             display.draw_rectangle(rect, color::dim_background);
    //         }
    //         display.draw_button(button)
    //     }

    //     display.draw_button(&layout.back_button);
    //     display.draw_button(&layout.apply_button);
    // }
}
