#![cfg_attr(rustfmt, rustfmt_skip)]

use crate::color::Color;

// Source: https://personal.sron.nl/~pault/
//
// This palette should be accessible to people with various forms of
// colour blindness.

pub const BLACK: Color = Color { r: 0, g: 0, b: 0 };
pub const WHITE: Color = Color {r: 255, g: 255, b: 255};

pub const GREY: Color = Color {r: 187, g: 187, b: 187};
pub const LIGHT_GREY: Color = GREY;
pub const DARK_GREY: Color = GREY;

pub const BLUE: Color = Color {r: 0, g: 119, b: 187};
pub const BRIGHT_BLUE: Color = Color {r: 51, g: 187, b: 238};
pub const DIM_BLUE: Color = BLUE;

pub const NATURAL_GREEN: Color = Color {r: 34, g: 136, b: 51};
pub const BRIGHT_GREEN: Color = Color {r: 0, g: 153, b: 136};
pub const DIM_GREEN: Color = NATURAL_GREEN;
pub const DARK_GREEN: Color = NATURAL_GREEN;

pub const RED: Color = Color {r: 204, g: 51, b: 17};
pub const DARK_RED: Color = RED;

pub const PURPLE: Color = Color {r: 170, g: 51, b: 119};

pub const BROWN: Color = Color {r: 221, g: 170, b: 51};
pub const DARK_BROWN: Color = BROWN;

pub const ORANGE: Color = Color {r: 238, g: 119, b: 51};

pub const NEON_GREEN: Color = BRIGHT_GREEN;
pub const NEON_PURPLE: Color = Color {r: 238, g: 51, b: 119};

pub const VERY_LIGHT_BLUE: Color = WHITE;
pub const VERY_LIGHT_RED: Color = WHITE;
pub const VERY_LIGHT_GREEN: Color = WHITE;
pub const VERY_LIGHT_PURPLE: Color = WHITE;
pub const VERY_LIGHT_YELLOW: Color = WHITE;
pub const VERY_LIGHT_BROWN: Color = WHITE;
