use std::collections::HashMap;

use monster::Monster;
use point::Point;
use level::Level;

pub struct Path;

impl Path {
    pub fn find(from: Point, to: Point, level: &Level) -> Path {
        unimplemented!()
    }

    /// The number of steps to necessary to reach the destination. If
    /// no path was found, it is `0`.
    pub fn len(&self) -> usize {
        unimplemented!()
    }
}

impl Iterator for Path {
    type Item = Point;

    fn next(&mut self) -> Option<Self::Item> {
        unimplemented!()
    }
}
