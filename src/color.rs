use serde::{Deserialize, Serialize};

#[derive(Debug, Copy, Clone, PartialEq, Eq, Serialize, Deserialize)]
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
        ColorAlpha { rgb: self, alpha }
    }
}

impl Default for Color {
    fn default() -> Self {
        Self {
            r: 255,
            g: 0,
            b: 255,
        }
    }
}

impl Into<ColorAlpha> for Color {
    fn into(self) -> ColorAlpha {
        self.alpha(255)
    }
}

impl Into<egui::Rgba> for Color {
    fn into(self) -> egui::Rgba {
        let color: ColorAlpha = self.into();
        egui::Rgba::from_rgba_premultiplied(
            color.rgb.r as f32 / 255.0,
            color.rgb.g as f32 / 255.0,
            color.rgb.b as f32 / 255.0,
            color.alpha as f32 / 255.0,
        )
    }
}

impl Into<egui::Color32> for Color {
    fn into(self) -> egui::Color32 {
        let color: ColorAlpha = self.into();
        egui::Color32::from_rgba_premultiplied(color.rgb.r, color.rgb.g, color.rgb.b, color.alpha)
    }
}

pub const BLACK: Color = Color { r: 0, g: 0, b: 0 };
pub const WHITE: Color = Color {
    r: 255,
    g: 255,
    b: 255,
};

pub const INVISIBLE: ColorAlpha = ColorAlpha {
    rgb: Color { r: 0, g: 0, b: 0 },
    alpha: 0,
};
