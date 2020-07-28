use crate::color::Color;

pub fn lerp_f32(from: f32, to: f32, t: f32) -> f32 {
    (1.0 - t) * from + t * to
}

pub fn lerp_u8(from: u8, to: u8, t: f32) -> u8 {
    lerp_f32(f32::from(from), f32::from(to), t) as u8
}

pub fn fade_color(from: Color, to: Color, progress: f32) -> Color {
    debug_assert!(progress >= 0.0);
    debug_assert!(progress <= 1.0);
    Color {
        r: lerp_u8(from.r, to.r, progress),
        g: lerp_u8(from.g, to.g, progress),
        b: lerp_u8(from.b, to.b, progress),
    }
}
