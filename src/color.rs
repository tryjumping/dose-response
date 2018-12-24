#![allow(non_upper_case_globals)]
#![cfg_attr(rustfmt, rustfmt_skip)]

//pub use palette::original::*;
//pub use palette::dawnbringer32::*;
pub use crate::palette::dawnbringer16::*;

use serde::{Deserialize, Serialize};


#[derive(Debug, Copy, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

#[derive(Debug, Copy, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct ColorAlpha {
    /// RGB components of a colour
    pub rgb: Color,

    /// Transparence value of the colour.
    /// `0`: fully transparent
    /// `255`: fully opaque
    pub alpha: u8,
}


impl Color {
    pub fn alpha(self, alpha: u8) -> ColorAlpha {
        ColorAlpha {
            rgb: self,
            alpha,
        }
    }
}


impl Into<ColorAlpha> for Color {
    fn into(self) -> ColorAlpha {
        self.alpha(255)
    }
}


pub const invisible: ColorAlpha = ColorAlpha { rgb: Color { r: 0, g: 0, b: 0 }, alpha: 0 };


// Game colours
pub const explored_background: Color = BLACK;
pub const unexplored_background: Color = BLACK;
pub const exhaustion_animation: Color = BLACK;
pub const fade_to_black_animation: Color = BLACK;

pub const gui_text: Color = WHITE;
pub const overdose_animation: Color = WHITE;
pub const player: Color = WHITE;
pub const victory_npc: Color = WHITE;
pub const signpost: Color = WHITE;

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

pub const strong_dose: Color = BRIGHT_BLUE;
pub const shattering_dose: Color = BRIGHT_BLUE;
pub const explosion: Color = BRIGHT_BLUE;

pub const window_edge: Color = DIM_BLUE;
pub const window_background: Color = BLACK;
pub const menu_highlight: Color = RED;
pub const dose_irresistible_background: Color = DIM_BLUE;

pub const tree_3: Color = NATURAL_GREEN;

pub const gui_progress_bar_bg: Color = DIM_GREEN;

pub const anxiety_progress_bar_fg: Color = RED;
pub const anxiety_progress_bar_bg: Color = BROWN;

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
