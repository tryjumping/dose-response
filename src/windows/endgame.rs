use color;
use engine::{Draw, TextMetrics, TextOptions};
use formula;
use player::CauseOfDeath;
use point::Point;
use rect::Rectangle;
use state::{Side, State};
use ui;


pub enum Action {
    NewGame,
    Help,
    Menu,
}


struct Layout {
    action_under_mouse: Option<Action>,
    rect_under_mouse: Option<Rectangle>,
}


pub struct Window;

impl Window {
    fn layout(&self, _state: &State, _metrics: &TextMetrics) -> Layout {
        let action_under_mouse = None;
        let rect_under_mouse = None;

        Layout {
            action_under_mouse,
            rect_under_mouse,
        }
    }

    pub fn render(&self, state: &State, metrics: &TextMetrics, drawcalls: &mut Vec<Draw>) {
        use self::CauseOfDeath::*;
        use ui::Text::*;

        let layout = self.layout(state, metrics);


        let cause_of_death = formula::cause_of_death(&state.player);
        let endgame_reason_text = if state.side == Side::Victory {
            // TODO: remove Side entirely for now.
            assert!(state.player.alive());
            assert!(cause_of_death.is_none());
            "You won!"
        } else {
            "You lost:"
        };

        let perpetrator = state.player.perpetrator.as_ref();

        let endgame_description = match (cause_of_death, perpetrator) {
            (Some(Exhausted), None) => "Exhausted".into(),
            (Some(Exhausted), Some(monster)) => format!("Exhausted because of `{}`", monster.glyph()),
            (Some(Overdosed), _) => "Overdosed".into(),
            (Some(LostWill), Some(monster)) => format!("Lost all Will due to `{}`", monster.glyph()),
            (Some(LostWill), None) => unreachable!(),
            (Some(Killed), Some(monster)) => format!("Defeated by `{}`", monster.glyph()),
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
        let carrying_doses_text = format!("Carrying {} doses", doses_in_inventory);
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

        let padding = Point::from_i32(1);
        let size = Point::new(37, 17) + (padding * 2);
        let top_left = Point {
            x: (state.display_size.x - size.x) / 2,
            y: 7,
        };

        let window_rect = Rectangle::from_point_and_size(top_left, size);

        drawcalls.push(Draw::Rectangle(window_rect, color::background));

        let rect = Rectangle::new(
            window_rect.top_left() + padding,
            window_rect.bottom_right() - padding,
        );

        ui::render_text_flow(&lines, rect, metrics, drawcalls);

        if let Some(rect) = layout.rect_under_mouse {
            drawcalls.push(Draw::Rectangle(rect, color::menu_highlight));
        }

        drawcalls.push(Draw::Text(
            rect.bottom_left(),
            "[N]ew Game       [?] Help       [Esc] Main Menu".into(),
            color::gui_text,
            TextOptions::align_center(rect.width()),
        ));

    }

    pub fn hovered(&self, state: &State, metrics: &TextMetrics) -> Option<Action> {
        self.layout(state, metrics).action_under_mouse
    }
}


fn endgame_tip(state: &State) -> String {
    use rand::Rng;
    use self::CauseOfDeath::*;
    let mut throwavay_rng = state.rng.clone();

    let overdosed_tips = &[
        "Using another dose when High will likely cause overdose early on.",
        "When you get too close to a dose, it will be impossible to resist.",
        "The `+`, `x` and `I` doses are much stronger. Early on, you'll likely overdose on them.",
    ];

    let food_tips = &[
        "Eat food (by pressing [1]) or use a dose to stave off withdrawal.",
    ];

    let hunger_tips = &[
        "Being hit by `h` will quickly get you into a withdrawal.",
        "The `h` monsters can swarm you.",
    ];

    let anxiety_tips = &[
        "Being hit by `a` reduces your Will. You lose when it reaches zero.",
    ];

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
        (Some(Overdosed), _) => throwavay_rng.choose(overdosed_tips).unwrap(),
        (Some(Exhausted), Some(_monster)) => throwavay_rng.choose(hunger_tips).unwrap(),
        (Some(Exhausted), None) => throwavay_rng.choose(food_tips).unwrap(),
        (Some(LostWill), Some(_monster)) => throwavay_rng.choose(anxiety_tips).unwrap(),
        _ => throwavay_rng.choose(&all_tips).unwrap(),
    };

    String::from(*selected_tip)
}
