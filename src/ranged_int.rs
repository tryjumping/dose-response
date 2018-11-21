use std::{
    cmp,
    ops::{Add, AddAssign, Sub, SubAssign},
};

use num_rational::{Ratio, Rational32};
use rand::Rng;

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

impl InclusiveRange {
    pub fn random<R: Rng>(self, rng: &mut R) -> i32 {
        rng.gen_range(self.0, self.1 + 1)
    }
}

/// A bounded, fractional value.
///
/// The value carries a specified minimum and maximum (always i32)
/// that it will always clamp to.
///
/// Internally, the value is a `Rational32` which means it can be a
/// non-integer, but still precise and consistent (without the
/// floating point weirdness).
#[derive(Copy, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Ranged {
    val: Rational32,
    min: Rational32,
    max: Rational32,
}

// NOTE: Custom formatter that's always on 1 line even when pretty-printing
impl ::std::fmt::Debug for Ranged {
    fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> Result<(), ::std::fmt::Error> {
        write!(f, "{} in <{}..{}>", self.val, self.min, self.max)
    }
}

impl Ranged {
    pub fn new<N: Into<Rational32>>(value: N, range: InclusiveRange) -> Self {
        assert!(range.0 <= range.1);
        let val = value.into();
        let min = Ratio::from_integer(range.0);
        let max = Ratio::from_integer(range.1);
        let val = cmp::max(val, min);
        let val = cmp::min(val, max);
        Ranged { val, min, max }
    }

    pub fn new_min(range: InclusiveRange) -> Self {
        Self::new(range.0, range)
    }

    pub fn new_max(range: InclusiveRange) -> Self {
        Self::new(range.1, range)
    }

    pub fn set_to_min(&mut self) {
        self.val = self.min().into()
    }

    pub fn set_to_max(&mut self) {
        self.val = self.max().into()
    }

    pub fn min(&self) -> i32 {
        self.min.to_integer()
    }

    pub fn max(&self) -> i32 {
        self.max.to_integer()
    }

    pub fn is_min(&self) -> bool {
        self.val == self.min
    }

    pub fn is_max(&self) -> bool {
        self.val == self.max
    }

    pub fn to_int(&self) -> i32 {
        self.val.to_integer()
    }

    pub fn middle(&self) -> i32 {
        (self.max() - self.min()) / 2
    }

    pub fn percent(&self) -> f32 {
        let length = self.max() as f32 - self.min() as f32;
        let value = *self.val.numer() as f32 / *self.val.denom() as f32;
        let result = value / length;
        assert!(result >= 0.0);
        assert!(result <= 1.0);
        result
    }
}

impl Add<Rational32> for Ranged {
    type Output = Ranged;

    fn add(self, other: Rational32) -> Self::Output {
        let range = InclusiveRange(self.min(), self.max());
        // NOTE: Ratio doesn't have checked_add so we do the check on
        // an int representation to detect any overflows. We can't use
        // the value though as it does not contain the fractional
        // portion.
        match self.val.to_integer().checked_add(other.to_integer()) {
            Some(_) => {
                let v = self.val + other;
                let new_val = if v > self.max {
                    self.max
                } else if v < self.min {
                    self.min
                } else {
                    v
                };
                Ranged::new(new_val, range)
            }
            None => {
                if other > Ratio::from_integer(0) {
                    Ranged::new_max(range)
                } else {
                    Ranged::new_min(range)
                }
            }
        }
    }
}

impl AddAssign<Rational32> for Ranged {
    fn add_assign(&mut self, other: Rational32) {
        *self = *self + other
    }
}

impl Add<i32> for Ranged {
    type Output = Ranged;

    fn add(self, other: i32) -> Self::Output {
        self + Ratio::from_integer(other)
    }
}

impl AddAssign<i32> for Ranged {
    fn add_assign(&mut self, other: i32) {
        *self = *self + other
    }
}

impl Sub<Rational32> for Ranged {
    type Output = Ranged;

    fn sub(self, other: Rational32) -> Self::Output {
        match other.numer().checked_neg() {
            Some(_negative_numerator) => self + (-other),
            None => {
                let mut result = self;
                result.set_to_max();
                result
            }
        }
    }
}

impl Sub<i32> for Ranged {
    type Output = Ranged;

    fn sub(self, other: i32) -> Self::Output {
        self - Ratio::from_integer(other)
    }
}

impl SubAssign<i32> for Ranged {
    fn sub_assign(&mut self, other: i32) {
        *self = *self - other
    }
}

#[cfg(test)]
mod test {
    use super::{InclusiveRange, Ranged};
    use std::i32::{MAX, MIN};

    #[test]
    fn new() {
        assert_eq!(
            Ranged::new(1, InclusiveRange(1, 1)),
            Ranged {
                val: 1.into(),
                min: 1.into(),
                max: 1.into(),
            }
        );
        assert_eq!(
            Ranged::new(3, InclusiveRange(-3, 3)),
            Ranged {
                val: 3.into(),
                min: (-3).into(),
                max: 3.into(),
            }
        );
        assert_eq!(
            Ranged::new(-3, InclusiveRange(-3, 3)),
            Ranged {
                val: (-3).into(),
                min: (-3).into(),
                max: 3.into(),
            }
        );
    }

    #[test]
    fn new_outside_range() {
        assert_eq!(
            Ranged::new(-1, InclusiveRange(0, 1)),
            Ranged::new(0, InclusiveRange(0, 1))
        );
        assert_eq!(
            Ranged::new(5, InclusiveRange(10, 20)),
            Ranged::new(10, InclusiveRange(10, 20))
        );
        assert_eq!(
            Ranged::new(10, InclusiveRange(1, 2)),
            Ranged::new(2, InclusiveRange(1, 2))
        );
    }

    #[test]
    fn adding_positive() {
        assert_eq!(
            Ranged::new(1, InclusiveRange(-5, 5)) + 3,
            Ranged::new(4, InclusiveRange(-5, 5))
        );
        assert_eq!(
            Ranged::new(1, InclusiveRange(-5, 5)) + 4,
            Ranged::new(5, InclusiveRange(-5, 5))
        );
        assert_eq!(
            Ranged::new(1, InclusiveRange(-5, 5)) + 5,
            Ranged::new(5, InclusiveRange(-5, 5))
        );
        assert_eq!(
            Ranged::new(1, InclusiveRange(-5, 5)) + 6,
            Ranged::new(5, InclusiveRange(-5, 5))
        );
        assert_eq!(
            Ranged::new(1, InclusiveRange(-5, 5)) + 2938,
            Ranged::new(5, InclusiveRange(-5, 5))
        );
        assert_eq!(
            Ranged::new(1, InclusiveRange(-5, 5)) + MAX,
            Ranged::new(5, InclusiveRange(-5, 5))
        );
    }

    #[test]
    fn adding_negative() {
        assert_eq!(
            Ranged::new(1, InclusiveRange(-5, 5)) + (-1),
            Ranged::new(0, InclusiveRange(-5, 5))
        );
        assert_eq!(
            Ranged::new(1, InclusiveRange(-5, 5)) + (-5),
            Ranged::new(-4, InclusiveRange(-5, 5))
        );
        assert_eq!(
            Ranged::new(1, InclusiveRange(-5, 5)) + (-6),
            Ranged::new(-5, InclusiveRange(-5, 5))
        );
        assert_eq!(
            Ranged::new(1, InclusiveRange(-5, 5)) + (-7),
            Ranged::new(-5, InclusiveRange(-5, 5))
        );
        assert_eq!(
            Ranged::new(1, InclusiveRange(-5, 5)) + (-9328),
            Ranged::new(-5, InclusiveRange(-5, 5))
        );
        assert_eq!(
            Ranged::new(1, InclusiveRange(-5, 5)) + MIN,
            Ranged::new(-5, InclusiveRange(-5, 5))
        );
    }

    #[test]
    fn subtracting_positive() {
        assert_eq!(
            Ranged::new(1, InclusiveRange(-5, 5)) - 1,
            Ranged::new(0, InclusiveRange(-5, 5))
        );
        assert_eq!(
            Ranged::new(1, InclusiveRange(-5, 5)) - 5,
            Ranged::new(-4, InclusiveRange(-5, 5))
        );
        assert_eq!(
            Ranged::new(1, InclusiveRange(-5, 5)) - 6,
            Ranged::new(-5, InclusiveRange(-5, 5))
        );
        assert_eq!(
            Ranged::new(1, InclusiveRange(-5, 5)) - 7,
            Ranged::new(-5, InclusiveRange(-5, 5))
        );
        assert_eq!(
            Ranged::new(1, InclusiveRange(-5, 5)) - 9328,
            Ranged::new(-5, InclusiveRange(-5, 5))
        );
        assert_eq!(
            Ranged::new(1, InclusiveRange(-5, 5)) - MAX,
            Ranged::new(-5, InclusiveRange(-5, 5))
        );
    }

    #[test]
    fn subtracting_negative() {
        assert_eq!(
            Ranged::new(1, InclusiveRange(-5, 5)) - (-3),
            Ranged::new(4, InclusiveRange(-5, 5))
        );
        assert_eq!(
            Ranged::new(1, InclusiveRange(-5, 5)) - (-4),
            Ranged::new(5, InclusiveRange(-5, 5))
        );
        assert_eq!(
            Ranged::new(1, InclusiveRange(-5, 5)) - (-5),
            Ranged::new(5, InclusiveRange(-5, 5))
        );
        assert_eq!(
            Ranged::new(1, InclusiveRange(-5, 5)) - (-6),
            Ranged::new(5, InclusiveRange(-5, 5))
        );
        assert_eq!(
            Ranged::new(1, InclusiveRange(-5, 5)) - (-2938),
            Ranged::new(5, InclusiveRange(-5, 5))
        );
        assert_eq!(
            Ranged::new(1, InclusiveRange(-5, 5)) - MIN,
            Ranged::new(5, InclusiveRange(-5, 5))
        );
    }

    #[test]
    fn add_assign_positive() {
        let mut a = Ranged::new(1, InclusiveRange(-5, 5));
        a += 3;
        assert_eq!(a, Ranged::new(4, InclusiveRange(-5, 5)));
        a += 1;
        assert_eq!(a, Ranged::new(5, InclusiveRange(-5, 5)));
        a += 1;
        assert_eq!(a, Ranged::new(5, InclusiveRange(-5, 5)));
        a += 23898923;
        assert_eq!(a, Ranged::new(5, InclusiveRange(-5, 5)));
        a += MAX;
        assert_eq!(a, Ranged::new(5, InclusiveRange(-5, 5)));
    }

    #[test]
    fn add_assign_negative() {
        let mut b = Ranged::new(1, InclusiveRange(-5, 5));
        b += -5;
        assert_eq!(b, Ranged::new(-4, InclusiveRange(-5, 5)));
        b += -1;
        assert_eq!(b, Ranged::new(-5, InclusiveRange(-5, 5)));
        b += -1;
        assert_eq!(b, Ranged::new(-5, InclusiveRange(-5, 5)));
        b += -23898923;
        assert_eq!(b, Ranged::new(-5, InclusiveRange(-5, 5)));
        b += MIN;
        assert_eq!(b, Ranged::new(-5, InclusiveRange(-5, 5)));
    }

    #[test]
    fn sub_assign_positive() {
        let mut a = Ranged::new(1, InclusiveRange(-5, 5));
        a -= 5;
        assert_eq!(a, Ranged::new(-4, InclusiveRange(-5, 5)));
        a -= 1;
        assert_eq!(a, Ranged::new(-5, InclusiveRange(-5, 5)));
        a -= 1;
        assert_eq!(a, Ranged::new(-5, InclusiveRange(-5, 5)));
        a -= 389832;
        assert_eq!(a, Ranged::new(-5, InclusiveRange(-5, 5)));
        a -= MAX;
        assert_eq!(a, Ranged::new(-5, InclusiveRange(-5, 5)));
    }

    #[test]
    fn sub_assign_negative() {
        let mut b = Ranged::new(1, InclusiveRange(-5, 5));
        b -= -3;
        assert_eq!(b, Ranged::new(4, InclusiveRange(-5, 5)));
        b -= -1;
        assert_eq!(b, Ranged::new(5, InclusiveRange(-5, 5)));
        b -= -1;
        assert_eq!(b, Ranged::new(5, InclusiveRange(-5, 5)));
        b -= -389832;
        assert_eq!(b, Ranged::new(5, InclusiveRange(-5, 5)));
        b -= MIN;
        assert_eq!(b, Ranged::new(5, InclusiveRange(-5, 5)));
    }

    #[test]
    fn percent() {
        assert_eq!(Ranged::new(0, InclusiveRange(0, 1)).percent(), 0.0);
        assert_eq!(Ranged::new(1, InclusiveRange(0, 1)).percent(), 1.0);

        assert_eq!(Ranged::new(0, InclusiveRange(0, 2)).percent(), 0.0);
        assert_eq!(Ranged::new(1, InclusiveRange(0, 2)).percent(), 0.5);
        assert_eq!(Ranged::new(2, InclusiveRange(0, 2)).percent(), 1.0);

        assert_eq!(Ranged::new(0, InclusiveRange(0, 10)).percent(), 0.0);
        assert_eq!(Ranged::new(1, InclusiveRange(0, 10)).percent(), 0.1);
        assert_eq!(Ranged::new(9, InclusiveRange(0, 10)).percent(), 0.9);
        assert_eq!(Ranged::new(10, InclusiveRange(0, 10)).percent(), 1.0);
    }
}
