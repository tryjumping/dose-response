#![allow(non_upper_case_globals)]
#![cfg_attr(rustfmt, rustfmt_skip)]

#[derive(Debug, Copy, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}


// Palette
pub const BLACK: Color = Color { r: 0, g: 0, b: 0 };
pub const WHITE: Color = Color { r: 255, g: 255, b: 255 };
pub const FULL_RED: Color = Color { r: 255, g: 0, b: 0 };  // elim?
pub const FULL_GREEN: Color = Color { r: 0, g: 255, b: 0 };  // elim?

pub const GREY: Color = Color { r: 95, g: 95, b: 95 };
pub const LIGHT_GREY: Color = Color { r: 223, g: 223, b: 223 };  // elim?
pub const DARK_GREY: Color = Color { r: 30, g: 30, b: 30 };  // elim?

pub const BLUE: Color = Color { r: 114, g: 126, b: 255 };
pub const BRIGHT_BLUE: Color = Color { r: 15, g: 255, b: 243 };
pub const DIM_BLUE: Color = Color { r: 0, g: 64, b: 64 };

pub const BRIGHT_GREEN: Color = Color { r: 63, g: 255, b: 63 };
pub const DIM_GREEN: Color = Color { r: 20, g: 133, b: 20 };
pub const NATURAL_GREEN: Color = Color { r: 0, g: 191, b: 0 };

pub const RED: Color = Color { r: 191, g: 0, b: 0 };
pub const PURPLE: Color = Color { r: 111, g: 63, b: 255 };  // elim?
pub const BROWN: Color = Color { r: 148, g: 113, b: 0 };

pub const FUNKY_RED: Color = Color { r: 161, g: 39, b: 113 };
pub const FUNKY_BLUE: Color = Color { r: 58, g: 217, b: 183 };


// Game colours
pub const background: Color = BLACK;
pub const exhaustion_animation: Color = BLACK;

pub const gui_text: Color = WHITE;
pub const overdose_animation: Color = WHITE;
pub const player: Color = WHITE;

pub const death_animation: Color = FULL_RED;  // could eliminate?

pub const gui_progress_bar_fg: Color = FULL_GREEN;  // eliminate?
pub const tree_2: Color = FULL_GREEN;

pub const voices: Color = GREY;
pub const shadows: Color = GREY;
pub const npc_dim: Color = GREY;
pub const dead_player: Color = GREY;

pub const empty_tile: Color = LIGHT_GREY;  // eliminate?

pub const dim_background: Color = DARK_GREY;  // eliminate/join with empty tile?

pub const dose: Color = BLUE;

pub const dose_glow: Color = BRIGHT_BLUE;
pub const shattering_dose: Color = BRIGHT_BLUE;
pub const explosion: Color = BRIGHT_BLUE;

pub const window_edge: Color = DIM_BLUE;
pub const menu_highlight: Color = DIM_BLUE;
pub const dose_background: Color = DIM_BLUE;

pub const tree_3: Color = BRIGHT_GREEN;

pub const gui_progress_bar_bg: Color = DIM_GREEN;

pub const tree_1: Color = NATURAL_GREEN;

pub const anxiety: Color = RED;
pub const npc_will: Color = RED;
pub const shattering_explosion: Color = RED;

pub const depression: Color = PURPLE;  // elim?
pub const npc_speed: Color = PURPLE;

pub const hunger: Color = BROWN;
pub const npc_mind: Color = BROWN;
pub const food: Color = BROWN;

pub const high: Color = FUNKY_BLUE;
pub const high_to: Color = FUNKY_RED;

//pub const npc_golden: Color = Color { r: 193, g: 193, b: 68 };
