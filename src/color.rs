#![allow(non_upper_case_globals)]
#![cfg_attr(rustfmt, rustfmt_skip)]

#[derive(Debug, Copy, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}


// Palette
// Haphazardly put together with the help of the DawnBringer 32bit palette:
// http://pixeljoint.com/forum/forum_posts.asp?TID=16247
pub const BLACK: Color = Color { r: 0, g: 0, b: 0 };
pub const WHITE: Color = Color { r: 255, g: 255, b: 255 };

pub const GREY: Color = Color { r: 132, g: 126, b: 135 };
pub const DARK_GREY: Color = Color { r: 50, g: 60, b: 57 };

pub const BLUE: Color = Color { r: 99, g: 155, b: 255 };
pub const BRIGHT_BLUE: Color = Color { r: 95, g: 205, b: 228 };
pub const DIM_BLUE: Color = Color { r: 34, g: 32, b: 52 };

pub const BRIGHT_GREEN: Color = Color { r: 153, g: 229, b: 80 };
pub const DIM_GREEN: Color = Color { r: 75, g: 105, b: 47 };
pub const NATURAL_GREEN: Color = Color { r: 106, g: 190, b: 48 };

pub const RED: Color = Color { r: 172, g: 50, b: 50 };
pub const PURPLE: Color = Color { r: 118, g: 88, b: 138 };
pub const BROWN: Color = Color { r: 143, g: 86, b: 59 };

pub const FUNKY_RED: Color = Color { r: 215, g: 123, b: 186 };
pub const FUNKY_BLUE: Color = Color { r: 99, g: 155, b: 255 };


// Game colours
pub const background: Color = BLACK;
pub const exhaustion_animation: Color = BLACK;

pub const gui_text: Color = WHITE;
pub const overdose_animation: Color = WHITE;
pub const player: Color = WHITE;

pub const death_animation: Color = RED;

pub const gui_progress_bar_fg: Color = BRIGHT_GREEN;
pub const tree_2: Color = BRIGHT_GREEN;

pub const voices: Color = GREY;
pub const shadows: Color = GREY;
pub const npc_dim: Color = GREY;
pub const dead_player: Color = GREY;

pub const empty_tile: Color = GREY;

pub const dim_background: Color = DARK_GREY;

pub const dose: Color = BLUE;

pub const dose_glow: Color = BRIGHT_BLUE;
pub const shattering_dose: Color = BRIGHT_BLUE;
pub const explosion: Color = BRIGHT_BLUE;

pub const window_edge: Color = DIM_BLUE;
pub const menu_highlight: Color = DIM_BLUE;
pub const dose_background: Color = DIM_BLUE;

pub const tree_3: Color = NATURAL_GREEN;

pub const gui_progress_bar_bg: Color = DIM_GREEN;

pub const tree_1: Color = DIM_GREEN;

pub const anxiety: Color = RED;
pub const npc_will: Color = RED;
pub const shattering_explosion: Color = RED;

pub const depression: Color = PURPLE;
pub const npc_speed: Color = PURPLE;

pub const hunger: Color = BROWN;
pub const npc_mind: Color = BROWN;
pub const food: Color = BROWN;

pub const high: Color = FUNKY_BLUE;
pub const high_to: Color = FUNKY_RED;

//pub const npc_golden: Color = Color { r: 193, g: 193, b: 68 };
