#![allow(
    clippy::identity_op,
    clippy::wildcard_imports,
    clippy::match_bool,
    clippy::single_match_else,
    clippy::match_wildcard_for_single_variants,
    clippy::match_same_arms,
    clippy::default_trait_access,
    clippy::ptr_as_ptr,
    clippy::float_cmp,
    clippy::from_iter_instead_of_collect,
    clippy::collapsible_else_if,
    clippy::bool_assert_comparison
)]
#![deny(
    // Prevent panics
    clippy::expect_used,
    clippy::unwrap_used,
    clippy::ok_expect,
    clippy::panic,
    clippy::match_on_vec_items,
    clippy::manual_strip,
    clippy::await_holding_refcell_ref
)]

macro_rules! throw {
    ($message:expr) => {
        return core::result::Result::Err(std::boxed::Box::new(crate::error::Error::new($message)))
    };
}

pub mod ai;
pub mod animation;
pub mod audio;
pub mod blocker;
pub mod color;
pub mod engine;
pub mod error;
pub mod formula;
pub mod game;
pub mod generators;
pub mod graphic;
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
