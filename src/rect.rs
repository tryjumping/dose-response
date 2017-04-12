use point::Point;


#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Rectangle {
    pub top_left: Point,
    pub bottom_right: Point,
}

impl Rectangle {

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
        self.bottom_right - self.top_left
    }

    pub fn top_left(self) -> Point {
        self.top_left
    }

    pub fn bottom_right(self) -> Point {
        self.bottom_right
    }

    /// Returns `true` if the point is within the areas specified by
    /// the rectangle. The mach is inclusive, so a `Rectangle`
    /// contains its `top_left` and `bottom_right` corners.
    pub fn contains(self, point: Point) -> bool {
        point >= self.top_left && point <= self.bottom_right
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
    type Item=Point;

    fn next(&mut self) -> Option<Self::Item> {
        if self.current.y > self.rect.bottom_right.y {
            None
        } else {
            let result = self.current;
            if self.current.x == self.rect.bottom_right.x {
                self.current = Point { x: self.rect.top_left.x, y: self.current.y + 1};
            } else {
                self.current += (1, 0);
            }
            Some(result)
        }
    }
}
