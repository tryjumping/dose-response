#![allow(non_upper_case_globals)]

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

pub static background: Color = Color{r: 0, g: 0, b: 0};
pub static dim_background: Color = Color{r: 30, g: 30, b: 30};
pub static death_animation: Color = Color{r: 255, g: 0, b: 0};
pub static gui_text: Color = Color{r: 255, g: 255, b: 255};
pub static gui_progress_bar_fg: Color = Color{r: 0, g: 255, b: 0};
pub static gui_progress_bar_bg: Color = Color{r: 20, g: 133, b: 20};
pub static anxiety: Color = Color{r: 191, g: 0, b: 0};
pub static depression: Color = Color{r: 111, g: 63, b: 255};
pub static hunger: Color = Color{r: 127, g: 101, b: 63};
pub static voices: Color = Color{r: 95, g: 95, b: 95};
pub static shadows: Color = Color{r: 95, g: 95, b: 95};
pub static player: Color = Color{r: 255, g: 255, b: 255};
pub static dead_player: Color = Color{r: 80, g: 80, b: 80};
pub static empty_tile: Color = Color{r: 223, g: 223, b: 223};
pub static dose: Color = Color{r: 114, g: 126, b: 255};
pub static dose_glow: Color = Color{r: 15, g: 255, b: 243};
pub static shattering_dose: Color = Color{r: 15, g: 255, b: 243};
pub static dose_background: Color = Color{r: 0, g: 64, b: 64};
pub static explosion: Color = Color{r: 15, g: 255, b: 243};
pub static food: Color = Color{r: 148, g: 113, b: 0};
pub static tree_1: Color = Color{r: 0, g: 191, b: 0};
pub static tree_2: Color = Color{r: 0, g: 255, b: 0};
pub static tree_3: Color = Color{r: 63, g: 255, b: 63};
pub static high: Color = Color{r: 58, g: 217, b: 183};
pub static high_to: Color = Color{r: 161, g: 39, b: 113};
