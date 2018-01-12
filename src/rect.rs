use point::Point;


#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Rectangle {
    top_left: Point,
    bottom_right: Point,
}

impl Rectangle {

    pub fn from_point_and_size(top_left: Point, size: Point) -> Self {
        assert!(size > (0, 0));
        Rectangle {
            top_left,
            bottom_right: top_left + size - (1, 1),
        }
    }

    /// Create a new rectangle defined by its middle point and
    /// (half-width, half-height).
    ///
    /// The result will have dimensions `half_size.x * 2, half_size.y
    /// * 2`.
    pub fn center(center: Point, half_size: Point) -> Self {
        assert!(half_size > (0, 0));
        Rectangle {
            top_left: center - half_size,
            bottom_right: center + half_size,
        }
    }

    pub fn dimensions(self) -> Point {
        self.bottom_right - self.top_left + (1, 1)
    }

    pub fn width(self) -> i32 {
        self.dimensions().x
    }

    /// Returns `true` if the point is within the areas specified by
    /// the rectangle. The mach is inclusive, so a `Rectangle`
    /// contains its `top_left` and `bottom_right` corners.
    pub fn contains(self, point: Point) -> bool {
        point >= self.top_left && point <= self.bottom_right
    }

    /// Returns `true` if the two rectangles have at least one `Point`
    /// in common, `false` otherwise.
    pub fn intersects(self, other: Rectangle) -> bool {
        let left = self.bottom_right().x < other.top_left().x;
        let right = self.top_left().x > other.bottom_right().x;
        let above = self.bottom_right().y < other.top_left().y;
        let below = self.top_left().y > other.bottom_right().y;

        // They intersect if self is neither all the way to the left, right,
        // above or below `other`:
        let result = !(left || right || above || below);
        debug_assert_eq!(result, self.slow_intersects(other));
        result
    }

    /// Same as `intersects` but slow because it tests all the points
    /// inside one rectangle againts the other.
    fn slow_intersects(self, other: Rectangle) -> bool {
        // TODO: remove this once we're confident the main `intersects`
        // fn is working correctly.
        other.points().any(|point| self.contains(point))
    }

    pub fn top_left(self) -> Point {
        self.top_left
    }

    pub fn bottom_right(self) -> Point {
        self.bottom_right
    }

    pub fn bottom_left(self) -> Point {
        Point::new(self.top_left.x, self.bottom_right.y)
    }

    pub fn points(self) -> RectangleIterator {
        RectangleIterator {
            rect: self,
            current: self.top_left,
        }
    }
}

pub struct RectangleIterator {
    rect: Rectangle,
    current: Point,
}

impl Iterator for RectangleIterator {
    type Item = Point;

    fn next(&mut self) -> Option<Self::Item> {
        if self.current.y > self.rect.bottom_right.y {
            None
        } else {
            let result = self.current;
            if self.current.x == self.rect.bottom_right.x {
                self.current = Point {
                    x: self.rect.top_left.x,
                    y: self.current.y + 1,
                };
            } else {
                self.current += (1, 0);
            }
            Some(result)
        }
    }
}

#[cfg(test)]
mod tests{
    use super::Rectangle;
    use point::Point;

    #[test]
    fn smallest_rect() {
        let rect = Rectangle::from_point_and_size((0, 0).into(), (1, 1).into());
        assert_eq!(rect.dimensions(), Point::new(1, 1));
        assert_eq!(rect.points().collect::<Vec<_>>().len(), 1);

        let rect = Rectangle::from_point_and_size((5, 7).into(), (1, 1).into());
        assert_eq!(rect.dimensions(), Point::new(1, 1));
        assert_eq!(rect.points().collect::<Vec<_>>().len(), 1);
    }

    #[test]
    fn rect_size_2() {
        let rect = Rectangle::from_point_and_size((0, 0).into(), (2, 2).into());
        assert_eq!(rect.dimensions(), Point::new(2, 2));
        assert_eq!(rect.points().collect::<Vec<_>>().len(), 4);

        let rect = Rectangle::from_point_and_size((5, 7).into(), (2, 2).into());
        assert_eq!(rect.dimensions(), Point::new(2, 2));
        assert_eq!(rect.points().collect::<Vec<_>>().len(), 4);
    }

}
