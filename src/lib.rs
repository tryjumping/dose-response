pub mod ai;
pub mod animation;
pub mod audio;
pub mod blocker;
pub mod color;
pub mod engine;
pub mod graphic;
#[macro_use]
pub mod error;
pub mod formula;
pub mod game;
pub mod generators;
pub mod graphics;
pub mod item;
pub mod keys;
pub mod level;
pub mod metadata;
pub mod monster;
pub mod palette;
pub mod pathfinding;
pub mod player;
pub mod point;
pub mod random;
pub mod ranged_int;
pub mod rect;
pub mod render;
pub mod settings;
pub mod state;
pub mod stats;
pub mod timer;
pub mod ui;
pub mod util;
pub mod window;
pub mod windows;
pub mod world;

pub const WORLD_SIZE: point::Point = point::Point {
    x: 1_073_741_824,
    y: 1_073_741_824,
};
