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


impl Into<egui::Srgba> for Color {
    fn into(self) -> egui::Srgba {
        let color: ColorAlpha = self.into();
        egui::Srgba::new(color.rgb.r, color.rgb.g, color.rgb.b, color.alpha)
    }
}


pub const invisible: ColorAlpha = ColorAlpha { rgb: Color { r: 0, g: 0, b: 0 }, alpha: 0 };


// Game colours
pub const explored_background: Color = DARK_BROWN;
pub const unexplored_background: Color = BLACK;
pub const exhaustion_animation: Color = BLACK;
pub const fade_to_black_animation: Color = BLACK;

pub const gui_text: Color = WHITE;
pub const gui_text_inactive: Color = LIGHT_GREY;
pub const gui_button_background: Color = DARK_RED;
pub const overdose_animation: Color = WHITE;


pub const player_1: Color = VERY_LIGHT_BLUE;
pub const player_2: Color = VERY_LIGHT_RED;
pub const player_3: Color = VERY_LIGHT_GREEN;
pub const player_4: Color = VERY_LIGHT_PURPLE;
pub const player_5: Color = VERY_LIGHT_YELLOW;
pub const player_6: Color = VERY_LIGHT_BROWN;

pub const signpost: Color = WHITE;

pub const death_animation: Color = RED;

pub const gui_progress_bar_fg: Color = BRIGHT_GREEN;

pub const tree_1: Color = DARK_GREEN;
pub const tree_2: Color = BRIGHT_GREEN;
pub const tree_3: Color = NATURAL_GREEN;

pub const voices: Color = GREY;
pub const shadows: Color = GREY;
pub const npc_dim: Color = GREY;
pub const dead_player: Color = GREY;

//pub const empty_tile_ground: Color = BROWN;
pub const empty_tile_ground: Color = Color { r: 113, g: 78, b: 52 };
pub const empty_tile_leaves: Color = DIM_GREEN;
pub const empty_tile_twigs: Color = Color { r: 162, g: 97, b: 52};

pub const dim_background: Color = DARK_GREY;

pub const dose: Color = BLUE;

pub const strong_dose: Color = BRIGHT_BLUE;
pub const shattering_dose: Color = BRIGHT_BLUE;
pub const explosion: Color = BRIGHT_BLUE;

pub const window_edge: Color = DIM_BLUE;
pub const window_background: Color = BLACK;

// NOTE: Our old menus used it, but egui buttons have a different colour.
// Keeping it here for now just in case:
//pub const menu_highlight: Color = RED;

pub const dose_irresistible_background: Color = DIM_BLUE;

pub const gui_progress_bar_bg: Color = DIM_GREEN;

pub const anxiety_progress_bar_fg: Color = RED;
pub const anxiety_progress_bar_bg: Color = BROWN;

pub const anxiety: Color = RED;
pub const npc_will: Color = RED;
pub const shattering_explosion: Color = RED;

pub const depression: Color = PURPLE;
pub const npc_speed: Color = PURPLE;

pub const hunger: Color = ORANGE;

pub const npc_mind: Color = ORANGE;
pub const food: Color = ORANGE;

pub const high: Color = NEON_GREEN;
// NOTE: this neon pink is awesome, but it visually conflicts with the Anxiety red.
// If we can find a way to make it work, I'd love to bring it back.
// Maybe a thre-way colour circling with the green and purple?
//pub const high_to: Color = NEON_PINK;
pub const high_to: Color = NEON_PURPLE;
