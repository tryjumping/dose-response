use std::cmp;
use std::ops::{Add, AddAssign, Deref, Sub, SubAssign};


// TODO: Basically the reason we do this instead of `std::ops::Range`
// is that Range is non-copy. I'd also prefer to use the inclusive
// range, but that's not stabilised yet.
//
// So: if `std::ops::Range` ever gets `Copy`, use it instead. It will
// let us do `min..max+1` or `min...max` in the `formulas` module.
//
// NOTE: looking at the previous discussions, the std policy is to not
// implement Copy for iterators (and range is an iterator), because it
// can easily create footguns (you "move" an iterator, then call iter
// on the original nad it works but from the initial state). So we're
// probably stuck with this instead of the nicer syntax. Oh well.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct InclusiveRange(pub i32, pub i32);


#[derive(Copy, Clone, PartialEq, Eq)]
pub struct Ranged {
    val: i32,
    range: InclusiveRange,
}

// NOTE: Custom formatter that's always on 1 line even when pretty-printing
impl ::std::fmt::Debug for Ranged {
    fn fmt(&self, f: &mut ::std::fmt::Formatter) -> Result <(), ::std::fmt::Error> {
        let InclusiveRange(min, max) = self.range;
        write!(f, "{} in <{}..{}>", self.val, min, max)
    }
}

impl Ranged {
    pub fn new(value: i32, range: InclusiveRange) -> Self {
        assert!(range.0 <= range.1);
        let value = cmp::max(value, range.0);
        let value = cmp::min(value, range.1);
        Ranged {
            val: value,
            range: range,
        }
    }

    pub fn new_min(range: InclusiveRange) -> Self {
        Self::new(range.0, range)
    }

    pub fn new_max(range: InclusiveRange) -> Self {
        Self::new(range.1, range)
    }

    pub fn set_to_min(&mut self) {
        self.val = self.min()
    }

    pub fn min(&self) -> i32 {
        self.range.0
    }

    pub fn max(&self) -> i32 {
        self.range.1
    }

    pub fn is_min(&self) -> bool {
        self.val == self.min()
    }

    pub fn is_max(&self) -> bool {
        self.val == self.max()
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
                let new_val = if v > self.max() {
                    self.max()
                } else if v < self.min() {
                    self.min()
                } else {
                    v
                };
                Ranged::new(new_val, self.range)
            }
            None => {
                if other > 0 {
                    Ranged::new_max(self.range)
                } else {
                    Ranged::new_min(self.range)
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
            Ranged::new_max(self.range)
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
