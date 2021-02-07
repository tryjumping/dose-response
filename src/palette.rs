use crate::color::{Color, BLACK, WHITE};

use serde::{Deserialize, Serialize};

// pub mod original;
// pub mod dawnbringer32;
pub mod accessible;
pub mod dawnbringer16;

pub const TREE_COUNT: usize = 3;

#[derive(Clone, Serialize, Deserialize)]
pub struct Palette {
    pub gui_text: Color,
    pub gui_text_inactive: Color,
    pub gui_button_background: Color,
    pub gui_mind_progress_bar_fg: Color,
    pub gui_mind_progress_bar_bg: Color,
    pub gui_anxiety_progress_bar_fg: Color,
    pub gui_anxiety_progress_bar_bg: Color,
    pub gui_window_background: Color,
    pub gui_window_edge: Color,

    pub explored_background: Color,
    pub unexplored_background: Color,
    pub dim_background: Color,

    pub exhaustion_animation: Color,
    pub fade_to_black_animation: Color,
    pub overdose_animation: Color,
    pub death_animation: Color,

    pub high: Color,
    pub high_to: Color,

    pub player: [Color; 6],
    pub dead_player: Color,

    pub anxiety: Color,
    pub depression: Color,
    pub hunger: Color,
    pub voices: Color,
    pub shadows: Color,

    pub npc_dim: Color,
    pub npc_will: Color,
    pub npc_speed: Color,
    pub npc_mind: Color,

    pub dose: Color,
    pub strong_dose: Color,
    pub shattering_dose: Color,
    pub dose_irresistible_background: Color,
    pub explosion: Color,
    pub shattering_explosion: Color,

    pub food: Color,

    pub signpost: Color,

    pub tree: [Color; TREE_COUNT],

    pub empty_tile_ground: Color,
    pub empty_tile_leaves: Color,
    pub empty_tile_twigs: Color,
}

impl Palette {
    pub fn classic() -> Self {
        use dawnbringer16::*;
        Self {
            gui_text: WHITE,
            gui_text_inactive: LIGHT_GREY,
            gui_button_background: DARK_RED,
            gui_mind_progress_bar_fg: BRIGHT_GREEN,
            gui_mind_progress_bar_bg: DIM_GREEN,
            gui_anxiety_progress_bar_fg: RED,
            gui_anxiety_progress_bar_bg: FADED_REDDISH_BROWN,
            gui_window_background: BLACK,
            gui_window_edge: DIM_BLUE,

            explored_background: DARK_BROWN,
            unexplored_background: BLACK,
            dim_background: DARK_GREY,

            exhaustion_animation: BLACK,
            fade_to_black_animation: BLACK,
            death_animation: RED,
            overdose_animation: WHITE,

            high: NEON_GREEN,
            high_to: NEON_PURPLE,

            player: [
                VERY_LIGHT_BLUE,
                VERY_LIGHT_RED,
                VERY_LIGHT_GREEN,
                VERY_LIGHT_PURPLE,
                VERY_LIGHT_YELLOW,
                VERY_LIGHT_BROWN,
            ],

            dead_player: GREY,

            anxiety: RED,
            depression: PURPLE,
            hunger: ORANGE,
            voices: GREY,
            shadows: GREY,

            npc_dim: GREY,
            npc_will: RED,
            npc_speed: PURPLE,
            npc_mind: ORANGE,

            dose: BLUE,
            strong_dose: BRIGHT_BLUE,
            shattering_dose: BRIGHT_BLUE,
            dose_irresistible_background: DIM_BLUE,
            explosion: BRIGHT_BLUE,
            shattering_explosion: RED,

            food: ORANGE,

            signpost: WHITE,

            tree: [DARK_GREEN, BRIGHT_GREEN, NATURAL_GREEN],

            empty_tile_ground: BROWN,
            empty_tile_leaves: DIM_GREEN,
            empty_tile_twigs: LIGHT_BROWN,
        }
    }

    pub fn accessible() -> Self {
        todo!();
    }

    pub fn greyscale() -> Self {
        todo!();
    }

    /// Select one of the possible player colours based on the index.
    /// Return the first one if the index is out of bounds.
    pub fn player(&self, index: usize) -> Color {
        let default = self.player[0];
        *self.player.get(index).unwrap_or(&default)
    }

    pub fn tree(&self, index: usize) -> Color {
        let default = self.tree[0];
        *self.tree.get(index).unwrap_or(&default)
    }
}
