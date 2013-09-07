extern mod std;

use std::uint::range;
use std::rand::{random};

#[deriving(Rand)]
pub enum WorldItem {
    Empty,
    Tree,
    Dose,
    Monster,
}

pub fn forrest(w: uint, h: uint) -> ~[(int, int, WorldItem)] {
    let mut result: ~[(int, int, WorldItem)] = ~[];
    for range(0, w) |x| {
        for range(0, h) |y| {
            result.push((x as int, y as int, random()));
        }
    }
    result
}
