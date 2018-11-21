use crate::{item::Item, level::Tile, monster::Monster, point::Point};

pub mod forrest;

pub type GeneratedWorld = (Vec<(Point, Tile)>, Vec<Monster>, Vec<(Point, Item)>);
