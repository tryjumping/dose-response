#![allow(non_upper_case_globals)]
#![cfg_attr(rustfmt, rustfmt_skip)]

//pub use palette::dawnbringer32::*;
pub use palette::dawnbringer16::*;


#[derive(Debug, Copy, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}



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
