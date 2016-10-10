use std::cmp::{max, Ordering};
use std::fmt::{Display, Formatter, Error};
use std::ops::{Add, AddAssign, Div, Sub};

#[derive(Copy, Clone, Debug, Hash, PartialEq, Eq)]
pub struct Point {
    pub x: i32,
    pub y: i32,
}

impl Point {
    pub fn new(x: i32, y: i32) -> Self {
        Point{x: x, y: y}
    }

    pub fn distance<P: Into<Point>>(&self, other: P) -> f32 {
        let other = other.into();
        let a = (self.x - other.x).pow(2);
        let b = (self.y - other.y).pow(2);
        ((a + b) as f32).sqrt()
    }

    pub fn tile_distance<P: Into<Point>>(&self, other: P) -> i32 {
        let other = other.into();
        max((self.x - other.x).abs(), (self.y - other.y).abs())
    }

    pub fn square_area(&self, radius: i32) -> SquareArea {
        SquareArea::new(*self, radius)
    }

    pub fn circular_area(&self, radius: i32) -> CircularArea {
        CircularArea::new(*self, radius)
    }

    pub fn tuple(&self) -> (i32, i32) {
        (self.x, self.y)
    }
}

impl Into<Point> for (i32, i32) {
    fn into(self) -> Point {
        Point{ x: self.0, y: self.1 }
    }
}

impl Display for Point {
    fn fmt(&self, f: &mut Formatter) -> Result<(), Error> {
        write!(f, "({}, {})", self.x, self.y)
    }
}

impl Add for Point {
    type Output = Self;

    fn add(self, rhs: Self) -> Self {
        Point{ x: self.x + rhs.x, y: self.y + rhs.y }
    }
}

impl AddAssign for Point {
    fn add_assign(&mut self, rhs: Self) {
        *self = *self + rhs;
    }
}

impl Sub for Point {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self {
        self + Point::new(-rhs.x, -rhs.y)
    }
}

impl PartialOrd for Point {
    fn partial_cmp(&self, other: &Point) -> Option<Ordering> {
        if self == other {
            Some(Ordering::Equal)
        } else if (self.x < other.x) && (self.y < other.y) {
            Some(Ordering::Less)
        } else if (self.x > other.x) && (self.y > other.y) {
            Some(Ordering::Greater)
        } else {
            None
        }
    }
}

impl Add<(i32, i32)> for Point {
    type Output = Self;

    fn add(self, rhs: (i32, i32)) -> Self {
        let rhs: Point = rhs.into();
        self + rhs
    }
}

impl AddAssign<(i32, i32)> for Point {
    fn add_assign(&mut self, rhs: (i32, i32)) {
        let rhs: Point = rhs.into();
        *self = self.add(rhs);
    }
}

impl Sub<(i32, i32)> for Point {
    type Output = Self;

    fn sub(self, rhs: (i32, i32)) -> Self {
        let rhs: Point = rhs.into();
        self - rhs
    }
}

impl PartialEq<(i32, i32)> for Point {
    fn eq(&self, other: &(i32, i32)) -> bool {
        let other: Point = (*other).into();
        self == &other
    }
}

impl PartialOrd<(i32, i32)> for Point {
    fn partial_cmp(&self, other: &(i32, i32)) -> Option<Ordering> {
        let other: Point = (*other).into();
        self.partial_cmp(&other)
    }
}

impl Div<i32> for Point {
    type Output = Self;

    fn div(self, rhs: i32) -> Self {
        Point::new(self.x / rhs, self.y / rhs)
    }
}

pub fn point(x: i32, y: i32) -> Point {
    Point::new(x, y)
}


pub struct CircularArea {
    pos: Point,
    center: Point,
    radius: i32,
    initial_x: i32,
    max: Point
}

impl CircularArea {
    pub fn new<P: Into<Point>>(center: P, radius: i32) -> Self {
        let center = center.into();
        CircularArea {
            pos: center - (radius, radius),
            center: center,
            radius: radius,
            initial_x: center.x - radius,
            max: center + (radius, radius),
        }
    }
}

impl Iterator for CircularArea {
    type Item = Point;

    fn next(&mut self) -> Option<Point> {
        loop {
            if (self.pos.y > self.max.y) || (self.pos.x > self.max.x) {
                return None;
            }
            let current_point = self.pos;
            self.pos.x += 1;
            if self.pos.x > self.max.x {
                self.pos.x = self.initial_x;
                self.pos.y += 1;
            }
            if self.center.distance(current_point) < self.radius as f32 {
                return Some(current_point)
            } else {
                // Keep looping for another point
            }
        }
    }
}

pub struct SquareArea {
    pos: Point,
    min_x: i32,
    max: Point,
}

impl SquareArea {
    pub fn new<P: Into<Point>>(center: P, half_side: i32) -> Self {
        let center = center.into();
        SquareArea {
            pos: center - (half_side, half_side),
            min_x: center.x - half_side,
            max: center + (half_side, half_side),
        }
    }
}

impl Iterator for SquareArea {
    type Item = Point;

    fn next(&mut self) -> Option<Point> {
        if self.pos.y > self.max.y {
            return None
        }
        let current_point = self.pos;
        self.pos.x += 1;
        if self.pos.x > self.max.x {
            self.pos.y += 1;
            self.pos.x = self.min_x;
        }
        return Some(current_point)
    }
}

#[cfg(test)]
mod test {
    use std::iter::FromIterator;
    use std::f32::EPSILON;
    use super::{point, Point, SquareArea};

    #[test]
    fn test_tile_distance() {
        assert_eq!(point(0, 0).tile_distance((0, 0)), 0);

        assert_eq!(point(0, 0).tile_distance(( 1, 0)), 1);
        assert_eq!(point(0, 0).tile_distance((-1, 0)), 1);
        assert_eq!(point(0, 0).tile_distance(( 1, 1)), 1);
        assert_eq!(point(0, 0).tile_distance((-1, 1)), 1);
        assert_eq!(point(0, 0).tile_distance((0,  1)), 1);
        assert_eq!(point(0, 0).tile_distance((0, -1)), 1);
        assert_eq!(point(0, 0).tile_distance((1,  1)), 1);
        assert_eq!(point(0, 0).tile_distance((1, -1)), 1);

        assert_eq!(point(0, 0).tile_distance((2, 2)), 2);
        assert_eq!(point(0, 0).tile_distance((-2, -2)), 2);
        assert_eq!(point(0, 0).tile_distance((0, 2)), 2);
        assert_eq!(point(0, 0).tile_distance((2, 0)), 2);

        assert_eq!(point(-3, -3).tile_distance((10, 10)), 13);
        assert_eq!(point(-3, -3).tile_distance((5, -2)), 8);
    }

    #[test]
    fn test_euclidean_distance() {
        let actual = point(0, 0).distance((0, 0));
        let expected = 0.0;
        assert!((actual - expected).abs() <= EPSILON);

        let actual = point(0, 0).distance((10, 10));
        let expected = 14.142136;
        assert!((actual - expected).abs() <= EPSILON);

        let actual = point(0, 0).distance((10, -10));
        let expected = 14.142136;
        assert!((actual - expected).abs() <= EPSILON);

        let actual = point(0, 0).distance((-10, 10));
        let expected = 14.142136;
        assert!((actual - expected).abs() <= EPSILON);

        let actual = point(0, 0).distance((10, -10));
        let expected = 14.142136;
        assert!((actual - expected).abs() <= EPSILON);

        let actual = point(0, 0).distance((3, 4));
        let expected = 5.0;
        assert!((actual - expected).abs() <= EPSILON);

        let actual = point(0, 0).distance((-3, 4));
        let expected = 5.0;
        assert!((actual - expected).abs() <= EPSILON);

        let actual = point(0, 0).distance((3, -4));
        let expected = 5.0;
        assert!((actual - expected).abs() <= EPSILON);

        let actual = point(0, 0).distance((-3, -4));
        let expected = 5.0;
        assert!((actual - expected).abs() <= EPSILON);
}

    #[test]
    fn test_points_within_radius_of_zero() {
        let actual: Vec<Point> = FromIterator::from_iter(SquareArea::new((3, 3), 0));
        assert_eq!(actual, [(3, 3)]);
    }

    #[test]
    fn test_points_within_radius_of_one() {
        let actual: Vec<Point> = FromIterator::from_iter(SquareArea::new((0, 0), 1));
        let expected = [(-1, -1), (0, -1), (1, -1),
                        (-1,  0), (0,  0), (1,  0),
                        (-1,  1), (0,  1), (1,  1)];
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_points_within_radius_of_five() {
        let actual: Vec<Point> = FromIterator::from_iter(SquareArea::new((0, 0), 5));

        let mut expected = Vec::new();
        for y in -5..6 {
            for x in -5..6 {
                expected.push(Point{x: x, y: y});
            }
        }
        assert_eq!(actual, expected);
    }
}
