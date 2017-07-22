use item::Item;
use level::Tile;
use monster::Monster;
use point::Point;


pub mod forrest;


pub type GeneratedWorld = (
    Vec<(Point, Tile)>,
    Vec<Monster>,
    Vec<(Point, Item)>,
);
