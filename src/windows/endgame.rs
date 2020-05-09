use crate::color;
use crate::engine::{Display, TextMetrics, TextOptions};
use crate::formula;
use crate::player::CauseOfDeath;
use crate::point::Point;
use crate::rect::Rectangle;
use crate::state::{Side, State};
use crate::ui::{self, Button};

pub enum Action {
    NewGame,
    Help,
    Menu,
    Close,
}

struct Layout {
    window_rect: Rectangle,
    rect: Rectangle,
    action_under_mouse: Option<Action>,
    rect_under_mouse: Option<Rectangle>,
    new_game_button: Button,
    help_button: Button,
    menu_button: Button,
    close_button: Button,
    short: bool,
}

pub struct Window;

impl Window {
    fn layout(
        &self,
        state: &State,
        metrics: &dyn TextMetrics,
        display: &Display,
        top_level: bool,
    ) -> Layout {
        let short = display.size_without_padding().y < 26;

        let mut action_under_mouse = None;
        let mut rect_under_mouse = None;

        let padding = Point::from_i32(1);
        let height = if short {
            display.size_without_padding().y - (padding.y * 2)
        } else {
            17
        };
        let size = Point::new(37, height) + (padding * 2);
        let top_left = Point {
            x: (display.size_without_padding().x - size.x) / 2,
            y: if short { 0 } else { 7 },
        };

        let window_rect = Rectangle::from_point_and_size(top_left, size);

        let rect = Rectangle::new(
            window_rect.top_left() + padding + (1, 1),
            window_rect.bottom_right() - padding - (1, 1),
        );

        let new_game_button = Button::new(rect.bottom_left(), "[N]ew Game").align_left();

        let help_button = Button::new(rect.bottom_left(), "[?] Help").align_center(rect.width());

        let menu_button = Button::new(rect.bottom_right(), "[Esc] Main Menu").align_right();

        let text_rect = metrics.button_rect(&new_game_button);
        if text_rect.contains(state.mouse.tile_pos) {
            action_under_mouse = Some(Action::NewGame);
            rect_under_mouse = Some(text_rect);
        }

        let text_rect = metrics.button_rect(&help_button);
        // NOTE(shadower): This is a fixup for the discrepancy between
        // the text width in pixels and how it maps to the tile
        // coordinates. It just looks better 1 tile wider.
        let text_rect = Rectangle::new(text_rect.top_left(), text_rect.bottom_right() + (1, 0));
        if text_rect.contains(state.mouse.tile_pos) {
            action_under_mouse = Some(Action::Help);
            rect_under_mouse = Some(text_rect);
        }

        let text_rect = metrics.button_rect(&menu_button);
        if text_rect.contains(state.mouse.tile_pos) {
            action_under_mouse = Some(Action::Menu);
            rect_under_mouse = Some(text_rect);
        }

        let close_button = {
            let text = format!("[Esc] Close");
            let mut button = Button::new(window_rect.top_right() - (1, 0), &text);
            button.text_options = TextOptions::align_right();
            let button_rect = metrics.button_rect(&button);
            if button_rect.contains(state.mouse.tile_pos) {
                action_under_mouse = Some(Action::Close);
                rect_under_mouse = Some(button_rect);
            }
            button
        };

        if !top_level {
            action_under_mouse = None;
            rect_under_mouse = None;
        }

        Layout {
            window_rect,
            rect,
            action_under_mouse,
            rect_under_mouse,
            new_game_button,
            help_button,
            menu_button,
            close_button,
            short,
        }
    }

    pub fn render(
        &self,
        state: &State,
        metrics: &dyn TextMetrics,
        display: &mut Display,
        top_level: bool,
    ) {
        use self::CauseOfDeath::*;
        use crate::ui::Text::*;

        let layout = self.layout(state, metrics, display, top_level);

        let cause_of_death = formula::cause_of_death(&state.player);

        let endgame_reason_text = if state.side == Side::Victory {
            if !state.player.alive() {
                log::warn!("The player appears to be dead on victory screen.");
            }
            if cause_of_death.is_some() {
                log::warn!("The player has active cause of dead on victory screen.");
            }
            "You won!"
        } else {
            "You lost:"
        };

        let perpetrator = state.player.perpetrator.as_ref();

        let endgame_description = match (cause_of_death, perpetrator) {
            (Some(Exhausted), None) => "Exhausted".into(),
            (Some(Exhausted), Some(monster)) => format!("Exhausted because of {}", monster.name(),),
            (Some(Overdosed), _) => "Overdosed".into(),
            (Some(LostWill), Some(monster)) => format!("Lost all Will due to {}", monster.name(),),
            (Some(LostWill), None) => {
                log::error!("Lost all will without any apparent cause.");
                format!("Lost all will")
            }
            (Some(Killed), Some(monster)) => format!("Defeated by {}", monster.name()),
            (Some(Killed), None) => {
                log::error!("Player lost without an apparent cause.");
                format!("Lost")
            }
            (None, _) => "".into(), // Victory
        };

        let doses_in_inventory = state
            .player
            .inventory
            .iter()
            .filter(|item| item.is_dose())
            .count();

        let turns_text = format!("Turns: {}", state.turn);
        let carrying_doses_text = if state.player_picked_up_a_dose {
            format!("Carrying {} doses", doses_in_inventory)
        } else {
            "You've never managed to save a dose for a later fix.".to_string()
        };
        let high_streak_text = format!(
            "Longest High streak: {} turns",
            state.player.longest_high_streak
        );

        let oneline_reason = format!("{} {}", endgame_reason_text, endgame_description);
        let mut lines = vec![];

        if layout.short {
            lines.push(Centered(&oneline_reason));
        } else {
            lines.push(Centered(endgame_reason_text));
            lines.push(Centered(&endgame_description));
        }

        let empty_line = if layout.short { 0 } else { 1 };
        let empty_block_lines = if layout.short { 1 } else { 2 };

        lines.extend(&[
            EmptySpace(empty_block_lines),
            Centered(&turns_text),
            EmptySpace(empty_line),
            Centered(&high_streak_text),
            EmptySpace(empty_line),
            Centered(&carrying_doses_text),
            EmptySpace(empty_block_lines),
        ]);

        let tip_text = format!("Tip: {}", endgame_tip(state));
        if state.side != Side::Victory {
            // Show some game tip, but not if the player just won
            lines.push(Paragraph(&tip_text));
            lines.push(EmptySpace(empty_block_lines));
        }

        display.draw_rectangle(layout.window_rect, color::window_edge);

        display.draw_rectangle(
            Rectangle::new(
                layout.window_rect.top_left() + (1, 1),
                layout.window_rect.bottom_right() - (1, 1),
            ),
            color::window_background,
        );

        ui::render_text_flow(&lines, layout.rect, 0, metrics, display);

        if let Some(rect) = layout.rect_under_mouse {
            display.draw_rectangle(rect, color::menu_highlight);
        }

        display.draw_button(&layout.new_game_button);
        display.draw_button(&layout.help_button);
        display.draw_button(&layout.menu_button);
        display.draw_button(&layout.close_button);
    }

    pub fn hovered(
        &self,
        state: &State,
        metrics: &dyn TextMetrics,
        display: &Display,
        top_level: bool,
    ) -> Option<Action> {
        self.layout(state, metrics, display, top_level)
            .action_under_mouse
    }
}

fn endgame_tip(state: &State) -> String {
    use self::CauseOfDeath::*;
    let throwavay_rng = &mut state.rng.clone();

    let overdosed_tips = &[
        "Using another dose when High will likely cause overdose early on.",
        "When you get too close to a dose, it will be impossible to resist.",
        "The `+`, `x` and `I` doses are much stronger. Early on, you'll likely overdose on them.",
    ];

    let food_tips = &["Eat food (by pressing [1]) or use a dose to stave off withdrawal."];

    let hunger_tips = &[
        "Being hit by `h` will quickly get you into a withdrawal.",
        "The `h` monsters can swarm you.",
    ];

    let anxiety_tips = &["Being hit by `a` reduces your Will. You lose when it reaches zero."];

    let unsorted_tips = &[
        "As you use doses, you slowly build up tolerance.",
        "Even the doses of the same kind can have different strength. Their purity varies.",
        "Directly confronting `a` will slowly increase your Will.",
        "The other characters won't talk to you while you're High.",
        "Bumping to another person sober will give you a bonus.",
        "The `D` monsters move twice as fast as you. Be careful.",
    ];

    let all_tips = overdosed_tips
        .iter()
        .chain(food_tips)
        .chain(hunger_tips)
        .chain(anxiety_tips)
        .chain(unsorted_tips)
        .collect::<Vec<_>>();

    let cause_of_death = formula::cause_of_death(&state.player);
    let perpetrator = state.player.perpetrator.as_ref();
    let selected_tip = match (cause_of_death, perpetrator) {
        (Some(Overdosed), _) => *throwavay_rng.choose(overdosed_tips).unwrap(),
        (Some(Exhausted), Some(_monster)) => *throwavay_rng.choose(hunger_tips).unwrap(),
        (Some(Exhausted), None) => *throwavay_rng.choose(food_tips).unwrap(),
        (Some(LostWill), Some(_monster)) => *throwavay_rng.choose(anxiety_tips).unwrap(),
        _ => *throwavay_rng.choose(&all_tips).unwrap(),
    };

    String::from(selected_tip)
}
