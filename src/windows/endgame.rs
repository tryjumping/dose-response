use crate::color;
use crate::engine::{Display, TextMetrics};
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
}

struct Layout {
    window_rect: Rectangle,
    rect: Rectangle,
    action_under_mouse: Option<Action>,
    rect_under_mouse: Option<Rectangle>,
    new_game_button: Button,
    help_button: Button,
    menu_button: Button,
}

pub struct Window;

impl Window {
    fn layout(&self, state: &State, metrics: &dyn TextMetrics, display: &Display) -> Layout {
        let mut action_under_mouse = None;
        let mut rect_under_mouse = None;

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

        Layout {
            window_rect,
            rect,
            action_under_mouse,
            rect_under_mouse,
            new_game_button,
            help_button,
            menu_button,
        }
    }

    pub fn render(&self, state: &State, metrics: &dyn TextMetrics, display: &mut Display) {
        use self::CauseOfDeath::*;
        use crate::ui::Text::*;

        let layout = self.layout(state, metrics, display);

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
            (Some(Exhausted), Some(monster)) => format!(
                "Exhausted because of {} ({})",
                monster.name(),
                monster.glyph()
            ),
            (Some(Overdosed), _) => "Overdosed".into(),
            (Some(LostWill), Some(monster)) => format!(
                "Lost all Will due to {} ({})",
                monster.name(),
                monster.glyph()
            ),
            (Some(LostWill), None) => unreachable!(),
            (Some(Killed), Some(monster)) => {
                format!("Defeated by {} ({})", monster.name(), monster.glyph())
            }
            (Some(Killed), None) => unreachable!(),
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
        let tip_text = format!("Tip: {}", endgame_tip(state));

        let lines = vec![
            Centered(endgame_reason_text),
            Centered(&endgame_description),
            EmptySpace(2),
            Centered(&turns_text),
            Empty,
            Centered(&high_streak_text),
            Empty,
            Centered(&carrying_doses_text),
            EmptySpace(2),
            Paragraph(&tip_text),
            EmptySpace(2),
        ];

        display.draw_rectangle(layout.window_rect, color::window_background);

        ui::render_text_flow(&lines, layout.rect, metrics, display);

        if let Some(rect) = layout.rect_under_mouse {
            display.draw_rectangle(rect, color::menu_highlight);
        }

        display.draw_button(&layout.new_game_button);
        display.draw_button(&layout.help_button);
        display.draw_button(&layout.menu_button);
    }

    pub fn hovered(
        &self,
        state: &State,
        metrics: &dyn TextMetrics,
        display: &Display,
    ) -> Option<Action> {
        self.layout(state, metrics, display).action_under_mouse
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
