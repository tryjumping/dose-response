use crate::{
    audio::{Audio, Effect},
    engine::Display,
    game::RunningState,
    gamepad::Gamepad,
    keys::KeyCode,
    state::State,
    ui,
};

use std::{
    fmt::{Display as FmtDisplay, Error, Formatter},
    time::Duration,
};

use egui::{self, ScrollArea, Ui};

use serde::{Deserialize, Serialize};

#[derive(Copy, Clone)]
pub enum Action {
    NextPage,
    PrevPage,
    LineUp,
    LineDown,
    Close,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Page {
    DoseResponse,
    Controls,
    HowToPlay,
    Legend,
    Credits,
    About,
}

impl Page {
    pub fn prev(self) -> Option<Self> {
        use self::Page::*;
        match self {
            DoseResponse => None,
            Controls => Some(DoseResponse),
            HowToPlay => Some(Controls),
            Legend => Some(HowToPlay),
            Credits => Some(Legend),
            About => Some(Credits),
        }
    }

    pub fn next(self) -> Option<Self> {
        use self::Page::*;
        match self {
            DoseResponse => Some(Controls),
            Controls => Some(HowToPlay),
            HowToPlay => Some(Legend),
            Legend => Some(Credits),
            Credits => Some(About),
            About => None,
        }
    }
}

impl FmtDisplay for Page {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        use self::Page::*;
        let s = match *self {
            DoseResponse => "Dose Response",
            Controls => "Controls",
            HowToPlay => "How to play",
            Legend => "Legend",
            Credits => "Credits",
            About => "About Dose Response",
        };
        f.write_str(s)
    }
}

pub const OVERVIEW: &str = "Dose Response is a roguelike: every time you start a game, the map will be different. The items and monsters will be in new places. And when you lose, that's it -- you can't reload and try again. You start from the beginning, with a brand new map. Every life matters.

You can't learn the map (because it changes), but you can learn the world. How do the monsters work? What happens when you take two doses at the same time? What's that glowing thing around a dose? What is food good for?

You will lose quickly and often. That's normal. Learn from it! What went wrong? Is there anything you could have done better? Were you saving an item for later that could have helped you?

Each run takes 3 - 10 minutes so you won't lose that much anyway. Experiment!";

pub const CONTROLS_HEADER: &str = "You control the @ character. It moves just like the king in Chess: one step in any direction. That means up, down, left, right, but also diagonally.
";

pub const CONTROLS_FOOTER: &str = "Using items: you can use an item you're carrying (food and later on, doses) by clicking on it in the sidebar or pressing its number on the keyboard (not numpad -- that's for movement).";

pub const NUMPAD_TEXT: &str = r"Numpad:

Imagine your @ is in the middle (where [5] is) and you just pick a direction.";

pub const NUMPAD_CONTROLS: &str = r"7 8 9
 \|/
4-@-6
 /|\
1 2 3
";

pub const ARROW_TEXT: &str = r"Arrow keys:

You will need [Shift] and [Ctrl] for diagonal movement. [Shift] means up and [Ctrl] means down. You combine them with the [Left] and [Right] keys.";

pub const ARROW_CONTROLS: &str = r"Shift+Left  Up  Shift+Right
         \  |  /
       Left-@-Right
         /  |  \
Ctrl+Left  Down Ctrl+Right
";

pub const MODIFIER_KEYS: &str = r"Instead of [Ctrl], you can also use any of: [Alt], [command], [option] and [control]. Whether these keys exist and are available depends on your system.";

pub const VI_KEYS_TEXT: &str = r#"Vi keys:

Those map to the letters on your keyboard. This makes more sense if you've ever used the Vi text editor."#;

pub const VI_KEYS_CONTROLS: &str = r"y k u
 \|/
h-@-l
 /|\
b j n
";

pub const CONTROLLER: &str = r"Controller:

Move in all eight directions using the left thumb stick. You point the stick in the direction you want to move to and then press the A (Xbox-like controllers) or X (on the PlayStation-like controllers) button to actually move.

To use an item, press B (or Circle) to open the inventory menu, navigate to the item and press A (or X) again.

Navigating menus: you can use the D-pad as well as the left thumb stick.
";

pub const HOW_TO_PLAY: &str = r#"Your character is an addict. Stay long without using a Dose, and the game is over. Eat Food to remain sober for longer. Using a Dose or eating Food will also defeat nearby enemies.

If you step into the glow around a Dose, you can't resist even if it means Overdosing yourself. At the beginning, you will also Overdose by using a Dose when you're still High or using a Dose that's too strong. By using Doses you build up tolerance. You'll need stronger Doses later on.

Each enemy has their own way of harming you. The Depression moves twice as fast. The Anxiety will reduce your Will on each hit. When it reaches zero, you will lose.

You attack enemies by bumping into them. Just move into the monster's tile and your character will attack it.

To progress, your Will needs to get stronger. Defeat enough Anxieties to make it go up. The Dose or Food "explosions" don't count though! Higher Will shrinks the irresistible area around Doses. It also lets you pick them up!

If you see another player characters, they are friendly. They will give you a bonus and follow you around, but only while you're Sober. You can have only one bonus active at a time."#;

pub const LEGEND: &str = "Monsters:
Anxiety: takes Will away when it hits you. Defeat them to win the game.
Depression: moves twice as fast. You lose immediately when it hits you.
Hunger: summons other Hungers nearby. Reduces your mind state.
Hearing Voices: paralyzes you for three turns.
Seeing Shadows: makes you move randomly for three turns.

NPC: ignores you when High. Talk to them Sober for a bonus.

Items:
Food: prolongs being Sober or in a Withdrawal. Kills monsters around you.
Dose: makes you High. When you're High already, you'll likely Overdose.
Cardinal Dose: Destroys trees in the horizontal and vertical lines.
Diagonal Dose: Destroys trees in the diagonal lines.
Strong Dose: very strong Dose. Don't use it by accident.

Each Dose has a faint glow around it. If you step into it, you will not be able to resist.

When the glow disappears completely, you can pick the dose up and use it later. Don't lose Will if you're carrying doses though!";

pub const CREDITS_DEV: &str = "Design and development by Tomas Sedovic: https://tomas.sedovic.cz/";
pub const TOMAS_URL: &str = "https://tomas.sedovic.cz/";

pub const CREDITS_TILES: &str = "Tiles by VEXED: https://vexed.zone/";
pub const TILES_LICENSE: &str = "licensed under Creative Commons 0";
pub const VEXED_URL: &str = "https://vexed.zone/";

pub const CREDITS_FONT: &str = "Mononoki typeface by Matthias Tellen: https://github.com/madmalik";
pub const MONONOKI_URL: &str = "https://github.com/madmalik";
pub const FONT_LICENSE: &str =
    "Copyright (c) 2013, Matthias Tellen <matthias.tellen@googlemail.com>
licensed under the SIL Open Font License, Version 1.1";

pub const CREDITS_MUSIC: &str = r#"Music:
"Exit Exit" by P C III
License: Creative Commons Attribution (cc-by)

"earth2earth" by P C III
License: Creative Commons Attribution (cc-by)

"The Family Breaks" by P C III
License: Creative Commons Attribution (cc-by)
"#;

pub const CODE_LICENSE_ONELINE: &str =
    "licensed under GNU Affero General Public License 3 or later";

pub const CODE_LICENSE_BLOCK: &str = "Dose Response is a Free and Open Source software provided under the terms of GNU Affero General Public License version 3 or later. If you did not receive the license text with the program, you can read it here:";
pub const AGPL_URL: &str = "https://www.gnu.org/licenses/agpl-3.0.en.html";

pub const THIRD_PARTY_CODE_LICENSE: &str = "To see the license for the third-party code used in this game, open the third-party-licenses.html file.";

pub fn process(
    state: &mut State,
    ui: &mut Ui,
    gamepad: &Gamepad,
    display: &Display,
    audio: &mut Audio,
) -> RunningState {
    let mut visible = true;

    let mut action = None;

    let screen_size_px = display.screen_size_px;
    let window_size_px = [
        (screen_size_px.x - 150).min(1024) as f32,
        (screen_size_px.y - 350).min(600) as f32,
    ];
    let window_pos_px = [(screen_size_px.x as f32 - window_size_px[0]) / 2.0, 100.0];

    let scroll_delta = ui.ctx().input().scroll_delta;

    egui::Window::new(format!("{}", state.current_help_window))
        .open(&mut visible)
        .collapsible(false)
        .fixed_pos(window_pos_px)
        .fixed_size(window_size_px)
        .show(ui.ctx(), |ui| {
            let scroll_area = ScrollArea::vertical()
                .max_height(window_size_px[1]);
	    let scroll_area = if scroll_delta == egui::Vec2::ZERO {
		match state.help_starting_line {
		    Some(starting_line) => {
			scroll_area.scroll_offset((starting_line * 50) as f32)
		    }
		    None => scroll_area
		}
	    } else {
		state.help_starting_line = None;
		scroll_area
	    };
	    scroll_area.show(ui, |ui| {
                // NOTE: HACK: the 7px value hides the scrollbar on contents that doesn't overflow.
                ui.set_min_height(window_size_px[1] - 7.0);
                let copyright = format!("Copyright 2013-2024 {}", crate::metadata::AUTHORS);
                match state.current_help_window {
                    Page::DoseResponse => {
                        ui.label(OVERVIEW);
                    }

                    Page::Controls => {
                        ui.label(CONTROLS_HEADER);
                        ui.label(NUMPAD_TEXT);
                        ui.label("");
                        // NOTE: this is a hack for not having a
                        // way to center a label but it works:
                        ui.columns(1, |c| {
                            c[0].with_layout(
                                egui::Layout::top_down(egui::Align::Center),
                                |ui| {
                                    ui.label(NUMPAD_CONTROLS);
                                },
                            );
                        });
                        ui.label(ARROW_TEXT);
                        ui.label("");
                        ui.columns(1, |c| {
                            c[0].with_layout(
                                egui::Layout::top_down(egui::Align::Center),
                                |ui| {
                                    ui.label(ARROW_CONTROLS);
                                },
                            );
                        });
                        ui.label(MODIFIER_KEYS);
                        ui.label("");
                        ui.label(VI_KEYS_TEXT);
                        ui.label("");
                        ui.columns(1, |c| {
                            c[0].with_layout(
                                egui::Layout::top_down(egui::Align::Center),
                                |ui| {
                                    ui.label(VI_KEYS_CONTROLS);
                                },
                            );
                        });
                        ui.label(CONTROLS_FOOTER);
                        ui.label("");
                        ui.label(CONTROLLER);
                    }

                    Page::HowToPlay => {
                        ui.label(HOW_TO_PLAY);
                    }

                    Page::Legend => {
                        ui.label(LEGEND);
                    }

                    Page::Credits => {
                        ui.hyperlink_to(CREDITS_DEV, TOMAS_URL);
                        ui.label(copyright);
                        ui.label(CODE_LICENSE_ONELINE);
                        ui.label("");
                        ui.hyperlink_to(CREDITS_TILES, VEXED_URL);
                        ui.label(TILES_LICENSE);
                        ui.label("");
                        ui.hyperlink_to(CREDITS_FONT, MONONOKI_URL);
                        ui.label(FONT_LICENSE);
                        ui.label("");
                        ui.label(CREDITS_MUSIC);
                    }

                    Page::About => {
                        let version = format!(
                            "{} version: {}",
                            crate::metadata::TITLE,
                            crate::metadata::VERSION
                        );

                        ui.label(version);
                        ui.hyperlink_to(
                            format!("Homepage: {}", crate::metadata::HOMEPAGE),
                            crate::metadata::HOMEPAGE,
                        );
                        ui.label(copyright);

                        ui.label("");
                        ui.label(CODE_LICENSE_BLOCK);
                        ui.hyperlink(AGPL_URL);
                        ui.label("");
                        ui.label(THIRD_PARTY_CODE_LICENSE);
                        ui.label("");

                        if !crate::metadata::GIT_HASH.trim().is_empty() {
                            ui.label(format!("Git commit: {}", crate::metadata::GIT_HASH));
                        }

			let features = crate::metadata::FEATURES.replace(":", "\n* ");

			let configs = crate::metadata::CONFIGS.replace(":", "\n* ");

			ui.label(format!("Build profile: {}\nOptimisation level: {}\n\nBuild features: \n* {features}\n\nBuild configs: \n* {configs}\n\n", crate::metadata::PROFILE, crate::metadata::OPT_LEVEL));
                    }
                };
            });

            // TODO: looks like the separator is no longer being rendered??
            // NOTE: on linux, the separator is visible but super thin, almost invisible
            // on macOS, it's working just fine
            ui.separator();
            ui.columns(2, |c| {
                if let Some(text) = state.current_help_window.prev() {
                    if ui::button(&mut c[0], &format!("[<-] {}", text), true, &state.palette)
                        .clicked()
                    {
                        action = Some(Action::PrevPage);
                    }
                };

                if let Some(text) = state.current_help_window.next() {
                    c[1].with_layout(egui::Layout::top_down_justified(egui::Align::Max), |ui| {
                        if ui::button(ui, &format!("[->] {}", text), true, &state.palette).clicked()
                        {
                            action = Some(Action::NextPage);
                        }
                    });
                };
            });
        });

    if state.keys.matches_code(KeyCode::Esc) || state.mouse.right_clicked {
        action = Some(Action::Close);
    }

    if !visible {
        action = Some(Action::Close);
    }

    if action.is_none() {
        let stick_flicked_up = gamepad.left_stick_flicked && gamepad.left_stick_y > 0.0;
        let stick_flicked_down = gamepad.left_stick_flicked && gamepad.left_stick_y < 0.0;
        let stick_flicked_left = gamepad.left_stick_flicked && gamepad.left_stick_x < 0.0;
        let stick_flicked_right = gamepad.left_stick_flicked && gamepad.left_stick_x > 0.0;

        if state.keys.matches_code(KeyCode::Right) || stick_flicked_right {
            action = Some(Action::NextPage);
        } else if state.keys.matches_code(KeyCode::Left) || stick_flicked_left {
            action = Some(Action::PrevPage);
        } else if state.keys.matches_code(KeyCode::Up) || stick_flicked_up {
            action = Some(Action::LineUp);
        } else if state.keys.matches_code(KeyCode::Down) || stick_flicked_down {
            action = Some(Action::LineDown);
        }
    }

    if let Some(action) = action {
        audio.mix_sound_effect(Effect::Click, Duration::from_millis(0));
        match action {
            Action::NextPage => {
                let new_help_window = state
                    .current_help_window
                    .next()
                    .unwrap_or(state.current_help_window);
                state.current_help_window = new_help_window;
                state.help_starting_line = None;
            }

            Action::PrevPage => {
                let new_help_window = state
                    .current_help_window
                    .prev()
                    .unwrap_or(state.current_help_window);
                state.current_help_window = new_help_window;
                state.help_starting_line = None;
            }

            Action::LineUp => {
                if let Some(ref mut starting_line) = state.help_starting_line {
                    if *starting_line > 0 {
                        *starting_line -= 1;
                    }
                };
            }

            Action::LineDown => {
                let prev_line = state.help_starting_line.unwrap_or(0);
                state.help_starting_line = Some(prev_line + 1);
            }

            Action::Close => {
                state.window_stack.pop();
                return RunningState::Running;
            }
        }
    }

    RunningState::Running
}
