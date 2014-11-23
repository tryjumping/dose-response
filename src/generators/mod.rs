use item::Item;
use level::Tile;
use monster::Kind;
use point::Point;


pub mod forrest;


pub type GeneratedWorld = (Vec<(Point, Tile)>, Vec<(Point, Kind)>, Vec<(Point, Item)>);
