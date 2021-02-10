#![cfg_attr(rustfmt, rustfmt_skip)]

use crate::color::Color;

// Source: https://personal.sron.nl/~pault/
//
// This palette should be accessible to people with various forms of
// colour blindness.

pub const GREY: Color = Color {r: 187, g: 187, b: 187};
pub const DARK_GREY: Color = Color {r: 85, g: 85, b: 85};
pub const DARKEST_GREY: Color = Color {r: 45, g: 45, b: 45};

pub const BLUE: Color = Color {r: 0, g: 119, b: 187};
pub const CYAN: Color = Color {r: 51, g: 187, b: 238};

pub const GREEN: Color = Color {r: 34, g: 136, b: 51};
pub const TEAL: Color = Color {r: 0, g: 153, b: 136};

pub const RED: Color = Color {r: 204, g: 51, b: 17};

pub const PURPLE: Color = Color {r: 170, g: 51, b: 119};

pub const YELLOW: Color = Color {r: 221, g: 170, b: 51};

pub const ORANGE: Color = Color {r: 238, g: 119, b: 51};

pub const MAGENTA: Color = Color {r: 238, g: 51, b: 119};
