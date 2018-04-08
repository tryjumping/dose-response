use color::Color;

// Haphazardly put together with the help of the DawnBringer 16bit palette:
// http://pixeljoint.com/forum/forum_posts.asp?TID=12795
pub const BLACK: Color = Color { r: 0, g: 0, b: 0 };
pub const WHITE: Color = Color { r: 255, g: 255, b: 255 };

pub const GREY: Color = Color { r: 117, g: 113, b: 97 };
pub const DARK_GREY: Color = Color { r: 41, g: 39, b: 41 };

pub const BRIGHT_BLUE: Color = Color { r: 109, g: 194, b: 202 };
pub const BLUE: Color = BRIGHT_BLUE;
pub const DIM_BLUE: Color = Color { r: 48, g: 52, b: 109 };

pub const BRIGHT_GREEN: Color = Color { r: 109, g: 170, b: 44 };
pub const DIM_GREEN: Color = Color { r: 52, g: 101, b: 36 };
pub const NATURAL_GREEN: Color = BRIGHT_GREEN;

pub const RED: Color = Color { r: 208, g: 70, b: 72 };
pub const PURPLE: Color = Color { r: 218, g: 212, b: 94 };
pub const BROWN: Color = Color { r: 133, g: 76, b: 48 };

pub const FUNKY_RED: Color = Color { r: 210, g: 125, b: 44 };
pub const FUNKY_BLUE: Color = Color { r: 99, g: 155, b: 255 }; // TODO
