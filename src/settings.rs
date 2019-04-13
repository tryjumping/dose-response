use serde::{Deserialize, Serialize};

/// Settings the engine needs to carry.
///
/// Things such as the fullscreen/windowed display, font size, font
/// type, etc.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Settings {
    pub fullscreen: bool,
    pub tile_size: i32,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            fullscreen: false,
            tile_size: crate::engine::TILESIZE,
        }
    }
}
