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

struct PointsWithinRadius {
    x: int,
    y: int,
    initial_x: int,
    max_x: int,
    max_y: int,
}

impl Iterator<(int, int)> for PointsWithinRadius {
    fn next(&mut self) -> Option<(int, int)> {
        if (self.y > self.max_y) || (self.x > self.max_x) {
            return None;
        }
        let current_point = (self.x, self.y);
        self.x += 1;
        if self.x > self.max_x {
            self.x = self.initial_x;
            self.y += 1;
        }
        Some(current_point)
    }
}

pub fn points_within_radius<T: Point>(center: T, radius: int) -> PointsWithinRadius {
    let (center_x, center_y) = center.coordinates();
    PointsWithinRadius{
        x: center_x - radius,
        y: center_y - radius,
        initial_x: center_x - radius,
        max_x: center_x + radius,
        max_y: center_y + radius,
    }
}


#[cfg(test)]
mod test {
    use super::{Point, PointsWithinRadius, tile_distance, distance, points_within_radius};

    #[test]
    fn test_tile_distance() {
        assert_eq!(tile_distance((0, 0), (0, 0)), 0);
    }
}
