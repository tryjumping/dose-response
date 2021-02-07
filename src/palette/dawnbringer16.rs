#![cfg_attr(rustfmt, rustfmt_skip)]

use crate::color::Color;

// Haphazardly put together with the help of the DawnBringer 16bit palette:
// http://pixeljoint.com/forum/forum_posts.asp?TID=12795

pub const GREY: Color = Color {r: 117, g: 113, b: 97};
pub const LIGHT_GREY: Color = Color {r: 180, g: 180, b: 180};
pub const DARK_GREY: Color = Color {r: 41, g: 39, b: 41};

pub const BLUE: Color = BRIGHT_BLUE;
pub const BRIGHT_BLUE: Color = Color {r: 109, g: 194, b: 202};
pub const DIM_BLUE: Color = Color {r: 48, g: 52, b: 109};

pub const NATURAL_GREEN: Color = Color {r: 53, g: 178, b: 58};
pub const BRIGHT_GREEN: Color = Color {r: 109, g: 170, b: 44};
pub const DIM_GREEN: Color = Color {r: 52, g: 101, b: 36};
pub const DARK_GREEN: Color = Color { r: 0, g: 134, b: 7 };

pub const RED: Color = Color {r: 208, g: 70, b: 72};
pub const DARK_RED: Color = Color {r: 100, g: 10, b: 10};

pub const PURPLE: Color = Color {r: 218, g: 212, b: 94};

pub const BROWN: Color = Color {r: 133, g: 76, b: 48};
pub const DARK_BROWN: Color = Color {r: 39, g: 25, b: 14};

pub const ORANGE: Color = Color {r: 255, g: 136, b: 11};

pub const NEON_GREEN: Color = Color {r: 127, g: 255, b: 0};
pub const NEON_PURPLE: Color = Color {r: 138, g: 43, b: 226};

pub const VERY_LIGHT_BLUE: Color = Color {r: 204, g: 220, b: 255};
pub const VERY_LIGHT_RED: Color = Color {r: 255, g: 207, b: 207};
pub const VERY_LIGHT_GREEN: Color = Color {r: 204, g: 255, b: 208};
pub const VERY_LIGHT_PURPLE: Color = Color {r: 255, g: 211, b: 250};
pub const VERY_LIGHT_YELLOW: Color = Color {r: 251, g: 226, b: 224};
pub const VERY_LIGHT_BROWN: Color = Color {r: 235, g: 193, b: 174};
