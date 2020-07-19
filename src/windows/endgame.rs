use crate::{
    engine::{Display, TextMetrics},
    formula,
    player::CauseOfDeath,
    state::{Side, State},
};

use egui::{self, Ui};

pub enum Action {
    NewGame,
    Help,
    Menu,
    Close,
}

pub struct Window;

impl Window {
    pub fn process(
        &self,
        state: &State,
        ui: &mut Ui,
        _metrics: &dyn TextMetrics,
        _display: &Display,
    ) -> Option<Action> {
        use CauseOfDeath::*;
        let cause_of_death = formula::cause_of_death(&state.player);
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
        let endgame_reason_text = if state.side == Side::Victory {
            if !state.player.alive() {
                log::warn!("The player appears to be dead on victory screen.");
            }
            if cause_of_death.is_some() {
                log::warn!("The player has active cause of dead on victory screen.");
            }
            "You won!".into()
        } else {
            format!("You lost: {}", endgame_description)
        };

        let mut action = None;
        let mut window_is_open = true;

        egui::Window::new(&endgame_reason_text)
            .default_size([800.0, 600.0])
            .open(&mut window_is_open)
            .show(ui.ctx(), |ui| {
                ui.label(endgame_reason_text.clone());
                ui.label(format!("Turns: {}", state.turn));
                ui.label(format!(
                    "Longest High streak: {} turns",
                    state.player.longest_high_streak
                ));
                let carrying_doses_text = if state.player_picked_up_a_dose {
                    let doses_in_inventory = state
                        .player
                        .inventory
                        .iter()
                        .filter(|item| item.is_dose())
                        .count();
                    format!("Carrying {} doses", doses_in_inventory)
                } else {
                    "You've never managed to save a dose for a later fix.".to_string()
                };
                ui.label(carrying_doses_text);
                // Show some game tip, but not if the player just won
                if state.side != Side::Victory {
                    ui.label(format!("Tip: {}", endgame_tip(state)));
                }

                ui.separator();
                ui.horizontal(|ui| {
                    if ui.button("[N]ew Game").clicked {
                        action = Some(Action::NewGame);
                    };
                    if ui.button("[?] Help").clicked {
                        action = Some(Action::Help);
                    };
                    if ui.button("[Esc] Main Menu").clicked {
                        action = Some(Action::Menu);
                    };
                });
            });

        if !window_is_open {
            action = Some(Action::Close)
        };

        action
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
