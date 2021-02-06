use crate::color::Color;

use serde::{Deserialize, Serialize};

// pub mod original;
// pub mod dawnbringer32;
pub mod accessible;
pub mod dawnbringer16;

#[derive(Clone, Serialize, Deserialize)]
pub struct Palette {
    pub window_background: Color,
    pub window_edge: Color,
    pub gui_text: Color,
}

impl Palette {
    pub fn classic() -> Self {
        use dawnbringer16::*;
        Self {
            window_background: BLACK,
            window_edge: DIM_BLUE,
            gui_text: WHITE,
        }
    }

    pub fn accessible() -> Self {
        todo!();
    }

    pub fn greyscale() -> Self {
        todo!();
    }
}
