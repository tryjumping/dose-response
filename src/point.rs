use std::num::{Int, Float, SignedInt};
use std::cmp::{max};


pub type Point = (i32, i32);


pub fn tile_distance(p1: Point, p2: Point) -> i32 {
    let (x1, y1) = p1;
    let (x2, y2) = p2;
    max((x1 - x2).abs(), (y1 - y2).abs())
}

pub fn distance(p1: Point, p2: Point) -> f32 {
    let (x1, y1) = p1;
    let (x2, y2) = p2;
    let a = (x1 - x2).pow(2);
    let b = (y1 - y2).pow(2);
    ((a + b) as f32).sqrt()
}

struct PointsWithinRadius {
    x: i32,
    y: i32,
    center: Point,
    radius: i32,
    initial_x: i32,
    max_x: i32,
    max_y: i32,
}

impl Iterator for PointsWithinRadius {
    type Item = Point;

    fn next(&mut self) -> Option<Point> {
        loop {
            if (self.y > self.max_y) || (self.x > self.max_x) {
                return None;
            }
            let current_point = (self.x, self.y);
            self.x += 1;
            if self.x > self.max_x {
                self.x = self.initial_x;
                self.y += 1;
            }
            if distance(current_point, self.center) < self.radius as f32 {
                return Some(current_point)
            } else {
                // Keep looping for another point
            }
        }
    }
}

pub fn points_within_radius(center: Point, radius: i32) -> PointsWithinRadius {
    let (center_x, center_y) = center;
    PointsWithinRadius{
        x: center_x - radius,
        y: center_y - radius,
        center: center,
        radius: radius,
        initial_x: center_x - radius,
        max_x: center_x + radius,
        max_y: center_y + radius,
    }
}


#[cfg(test)]
mod test {
    use std::iter::FromIterator;
    use std::num::Float;
    use super::{tile_distance, distance, points_within_radius, Point};

    #[test]
    fn test_tile_distance() {
        assert_eq!(tile_distance((0, 0), (0, 0)), 0);

        assert_eq!(tile_distance((0, 0), ( 1, 0)), 1);
        assert_eq!(tile_distance((0, 0), (-1, 0)), 1);
        assert_eq!(tile_distance((0, 0), ( 1, 1)), 1);
        assert_eq!(tile_distance((0, 0), (-1, 1)), 1);
        assert_eq!(tile_distance((0, 0), (0,  1)), 1);
        assert_eq!(tile_distance((0, 0), (0, -1)), 1);
        assert_eq!(tile_distance((0, 0), (1,  1)), 1);
        assert_eq!(tile_distance((0, 0), (1, -1)), 1);

        assert_eq!(tile_distance((-3, -3), (10, 10)), 13);
        assert_eq!(tile_distance((-3, -3), (5, -2)), 8);
    }

    #[test]
    fn test_euclidean_distance() {
        let actual = distance((0, 0), (0, 0));
        let expected = 0.0;
        assert!((actual - expected).abs() <= Float::epsilon());

        let actual = distance((0, 0), (10, 10));
        let expected = 14.142136;
        assert!((actual - expected).abs() <= Float::epsilon());

        let actual = distance((0, 0), (10, -10));
        let expected = 14.142136;
        assert!((actual - expected).abs() <= Float::epsilon());

        let actual = distance((0, 0), (-10, 10));
        let expected = 14.142136;
        assert!((actual - expected).abs() <= Float::epsilon());

        let actual = distance((0, 0), (10, -10));
        let expected = 14.142136;
        assert!((actual - expected).abs() <= Float::epsilon());

        let actual = distance((0, 0), (3, 4));
        let expected = 5.0;
        assert!((actual - expected).abs() <= Float::epsilon());

        let actual = distance((0, 0), (-3, 4));
        let expected = 5.0;
        assert!((actual - expected).abs() <= Float::epsilon());

        let actual = distance((0, 0), (3, -4));
        let expected = 5.0;
        assert!((actual - expected).abs() <= Float::epsilon());

        let actual = distance((0, 0), (-3, -4));
        let expected = 5.0;
        assert!((actual - expected).abs() <= Float::epsilon());
}

    #[test]
    fn test_points_within_radius_of_zero() {
        let actual: Vec<Point> = FromIterator::from_iter(points_within_radius((3, 3), 0));
        assert_eq!(actual.as_slice(), [(3, 3)].as_slice());
    }

    #[test]
    fn test_points_within_radius_of_one() {
        let actual: Vec<Point> = FromIterator::from_iter(points_within_radius((3, 3), 1));
        let expected = [(2, 2), (3, 2), (4, 2),
                        (2, 3), (3, 3), (4, 3),
                        (2, 4), (3, 4), (4, 4)];
        assert_eq!(actual.as_slice(), expected.as_slice());
    }

    #[test]
    fn test_points_within_radius_of_five() {
        use std::iter::range_inclusive;

        let mut actual: Vec<Point> = FromIterator::from_iter(points_within_radius((0, 0), 5));

        let mut expected = Vec::new();
        for x in range_inclusive(-5, 5) {
            for y in range_inclusive(-5, 5) {
                expected.push((x, y));
            }
        }
        // the order is undefined so make sure we don't fail just because of ordering
        actual.sort();
        expected.sort();
        assert!(actual.as_slice() == expected.as_slice());
    }
}
