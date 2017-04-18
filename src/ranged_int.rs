use std::ops::{Add, AddAssign, Deref, Sub, SubAssign};


#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct Ranged {
    val: i32,
    _min: i32,
    _max: i32,
}

impl Ranged {
    pub fn new(value: i32, min: i32, max: i32) -> Self {
        assert!(min <= max);
        let val = if value < min {
            min
        } else if value > max {
            max
        } else {
            value
        };
        Ranged { val: val, _min: min, _max: max }
    }

    pub fn new_min(min: i32, max: i32) -> Self {
        Self::new(min, min, max)
    }

    pub fn new_max(min: i32, max: i32) -> Self {
        Self::new(max, min, max)
    }

    // NOTE: this is no longer being used -- lets just comment it out for now
    // pub fn set(&mut self, n: i32) {
    //     assert!(n >= self._min);
    //     assert!(n <= self._max);
    //     self.val = n;
    // }

    pub fn set_to_min(&mut self) {
        self.val = self._min
    }

    pub fn min(&self) -> i32 {
        self._min
    }

    pub fn max(&self) -> i32 {
        self._max
    }

    pub fn is_min(&self) -> bool {
        self.val == self._min
    }

    pub fn is_max(&self) -> bool {
        self.val == self._max
    }

    pub fn middle(&self) -> i32 {
        (self.max() - self.min()) / 2
    }

    pub fn percent(&self) -> f32 {
        let result = self.val as f32 / (self.max() - self.min()) as f32;
        assert!(result >= 0.0);
        assert!(result <= 1.0);
        result
    }
}

impl Add<i32> for Ranged {
    type Output = Ranged;

    fn add(self, other: i32) -> Self::Output {
        match self.val.checked_add(other) {
            Some(v) => {
                let new_val = if v > self._max {
                    self._max
                } else if v < self._min {
                    self._min
                } else {
                    v
                };
                Ranged::new(new_val, self._min, self._max)
            }
            None => {
                if other > 0 {
                    Ranged::new(self._max, self._min, self._max)
                } else {
                    Ranged::new(self._min, self._min, self._max)
                }
            }
        }
    }
}

impl AddAssign<i32> for Ranged {
    fn add_assign(&mut self, other: i32) {
        *self = self.clone() + other
    }
}

impl Sub<i32> for Ranged {
    type Output = Ranged;

    fn sub(self, other: i32) -> Self::Output {
        let (negated_val, overflowed) = other.overflowing_neg();
        if overflowed {
            Ranged::new(self._max, self._min, self._max)
        } else {
            self + negated_val
        }
    }
}

impl SubAssign<i32> for Ranged {
    fn sub_assign(&mut self, other: i32) {
        *self = self.clone() - other
    }
}

impl Deref for Ranged {
    type Target = i32;

    fn deref(&self) -> &i32 {
        &self.val
    }
}

#[cfg(test)]
mod test {
    use super::Ranged;
    use std::i32::{MAX, MIN};

    #[test]
    fn new() {
        assert_eq!(*Ranged::new(1, 1, 1), 1);
        assert_eq!(*Ranged::new(3, -3, 3), 3);
        assert_eq!(*Ranged::new(-3, -3, 3), -3);
    }

    #[test]
    fn new_outside_range() {
        assert_eq!(Ranged::new(-1, 0, 1), Ranged::new(0, 0, 1));
        assert_eq!(Ranged::new(5, 10, 20), Ranged::new(10, 10, 20));
        assert_eq!(Ranged::new(10, 1, 2), Ranged::new(2, 1, 2));
    }

    #[test]
    fn adding_positive() {
        assert_eq!(Ranged::new(1, -5, 5) + 3, Ranged::new(4, -5, 5));
        assert_eq!(Ranged::new(1, -5, 5) + 4, Ranged::new(5, -5, 5));
        assert_eq!(Ranged::new(1, -5, 5) + 5, Ranged::new(5, -5, 5));
        assert_eq!(Ranged::new(1, -5, 5) + 6, Ranged::new(5, -5, 5));
        assert_eq!(Ranged::new(1, -5, 5) + 2938, Ranged::new(5, -5, 5));
        assert_eq!(Ranged::new(1, -5, 5) + MAX, Ranged::new(5, -5, 5));
    }

    #[test]
    fn adding_negative() {
        assert_eq!(Ranged::new(1, -5, 5) + (-1), Ranged::new(0, -5, 5));
        assert_eq!(Ranged::new(1, -5, 5) + (-5), Ranged::new(-4, -5, 5));
        assert_eq!(Ranged::new(1, -5, 5) + (-6), Ranged::new(-5, -5, 5));
        assert_eq!(Ranged::new(1, -5, 5) + (-7), Ranged::new(-5, -5, 5));
        assert_eq!(Ranged::new(1, -5, 5) + (-9328), Ranged::new(-5, -5, 5));
        assert_eq!(Ranged::new(1, -5, 5) + MIN, Ranged::new(-5, -5, 5));
    }

    #[test]
    fn subtracting_positive() {
        assert_eq!(Ranged::new(1, -5, 5) - 1, Ranged::new(0, -5, 5));
        assert_eq!(Ranged::new(1, -5, 5) - 5, Ranged::new(-4, -5, 5));
        assert_eq!(Ranged::new(1, -5, 5) - 6, Ranged::new(-5, -5, 5));
        assert_eq!(Ranged::new(1, -5, 5) - 7, Ranged::new(-5, -5, 5));
        assert_eq!(Ranged::new(1, -5, 5) - 9328, Ranged::new(-5, -5, 5));
        assert_eq!(Ranged::new(1, -5, 5) - MAX, Ranged::new(-5, -5, 5));
    }

    #[test]
    fn subtracting_negative() {
        assert_eq!(Ranged::new(1, -5, 5) - (-3), Ranged::new(4, -5, 5));
        assert_eq!(Ranged::new(1, -5, 5) - (-4), Ranged::new(5, -5, 5));
        assert_eq!(Ranged::new(1, -5, 5) - (-5), Ranged::new(5, -5, 5));
        assert_eq!(Ranged::new(1, -5, 5) - (-6), Ranged::new(5, -5, 5));
        assert_eq!(Ranged::new(1, -5, 5) - (-2938), Ranged::new(5, -5, 5));
        assert_eq!(Ranged::new(1, -5, 5) - MIN, Ranged::new(5, -5, 5));
    }

    #[test]
    fn add_assign_positive() {
        let mut a = Ranged::new(1, -5, 5);
        a += 3;
        assert_eq!(a, Ranged::new(4, -5, 5));
        a += 1;
        assert_eq!(a, Ranged::new(5, -5, 5));
        a += 1;
        assert_eq!(a, Ranged::new(5, -5, 5));
        a += 23898923;
        assert_eq!(a, Ranged::new(5, -5, 5));
        a += MAX;
        assert_eq!(a, Ranged::new(5, -5, 5));
    }

    #[test]
    fn add_assign_negative() {
        let mut b = Ranged::new(1, -5, 5);
        b += -5;
        assert_eq!(b, Ranged::new(-4, -5, 5));
        b += -1;
        assert_eq!(b, Ranged::new(-5, -5, 5));
        b += -1;
        assert_eq!(b, Ranged::new(-5, -5, 5));
        b += -23898923;
        assert_eq!(b, Ranged::new(-5, -5, 5));
        b += MIN;
        assert_eq!(b, Ranged::new(-5, -5, 5));
    }

    #[test]
    fn sub_assign_positive() {
        let mut a = Ranged::new(1, -5, 5);
        a -= 5;
        assert_eq!(a, Ranged::new(-4, -5, 5));
        a -= 1;
        assert_eq!(a, Ranged::new(-5, -5, 5));
        a -= 1;
        assert_eq!(a, Ranged::new(-5, -5, 5));
        a -= 389832;
        assert_eq!(a, Ranged::new(-5, -5, 5));
        a -= MAX;
        assert_eq!(a, Ranged::new(-5, -5, 5));
    }

    #[test]
    fn sub_assign_negative() {
        let mut b = Ranged::new(1, -5, 5);
        b -= -3;
        assert_eq!(b, Ranged::new(4, -5, 5));
        b -= -1;
        assert_eq!(b, Ranged::new(5, -5, 5));
        b -= -1;
        assert_eq!(b, Ranged::new(5, -5, 5));
        b -= -389832;
        assert_eq!(b, Ranged::new(5, -5, 5));
        b -= MIN;
        assert_eq!(b, Ranged::new(5, -5, 5));
    }

    #[test]
    fn percent() {
        assert_eq!(Ranged::new(0, 0, 1).percent(), 0.0);
        assert_eq!(Ranged::new(1, 0, 1).percent(), 1.0);

        assert_eq!(Ranged::new(0, 0, 2).percent(), 0.0);
        assert_eq!(Ranged::new(1, 0, 2).percent(), 0.5);
        assert_eq!(Ranged::new(2, 0, 2).percent(), 1.0);

        assert_eq!(Ranged::new(0, 0, 10).percent(), 0.0);
        assert_eq!(Ranged::new(1, 0, 10).percent(), 0.1);
        assert_eq!(Ranged::new(9, 0, 10).percent(), 0.9);
        assert_eq!(Ranged::new(10, 0, 10).percent(), 1.0);
    }
}
