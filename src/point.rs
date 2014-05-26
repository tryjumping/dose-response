use std::num::{abs, pow, Float};
use std::cmp::{max};


pub trait Point {
    fn coordinates(&self) -> (int, int);
}

impl Point for (int, int) {
    fn coordinates(&self) -> (int, int) {
        *self
    }
}

pub fn tile_distance<P1: Point, P2: Point>(p1: P1, p2: P2) -> int {
    let (x1, y1) = p1.coordinates();
    let (x2, y2) = p2.coordinates();
    max(abs(x1 - x2), abs(y1 - y2))
}

pub fn distance<P1: Point, P2: Point>(p1: P1, p2: P2) -> f32 {
    let (x1, y1) = p1.coordinates();
    let (x2, y2) = p2.coordinates();
    let a = pow((x1 - x2) as f32, 2);
    let b = pow((y1 - y2) as f32, 2);
    (a + b).sqrt()
}
