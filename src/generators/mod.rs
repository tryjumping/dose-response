use crate::item::Item;
use crate::level::Tile;
use crate::monster::Monster;
use crate::point::Point;

pub mod forrest;

pub type GeneratedWorld = (Vec<(Point, Tile)>, Vec<Monster>, Vec<(Point, Item)>);
