use std::num::{abs, pow, Float};
use std::cmp::{max};

use components::Position;


pub fn distance(p1: &Position, p2: &Position) -> int {
    max(abs(p1.x - p2.x), abs(p1.y - p2.y))
}

pub fn precise_distance(p1: (int, int), p2: (int, int)) -> int {
    let (x1, y1) = p1;
    let (x2, y2) = p2;
    let a = pow(abs(x1 - x2) as f32, 2f32);
    let b = pow(abs(y1 - y2) as f32, 2f32);
    (a + b).sqrt().floor() as int
}
