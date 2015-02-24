use std::num::Int;
use std::ops::Deref;


pub struct RangedInt<T: Int> {
    val: T,
    _min: T,
    _max: T,
}

impl<T: Int> RangedInt<T> {
    pub fn new(value: T, (min, max): (T, T)) -> RangedInt<T> {
        assert!(min <= max && value >= min && value <= max);
        RangedInt {
            val: value,
            _min: min,
            _max: max,
        }
    }

    pub fn set(&mut self, n: T) {
        assert!((n >= self._min) && (n <= self._max));
        self.val = n;
    }

    pub fn add(&mut self, n: T) -> T {
        if let Some(v) = self.val.checked_add(n) {
            let new_val = if v > self._max {
                self._max
            } else if v < self._min {
                self._min
            } else {
                v
            };
            self.val = new_val;
        }
        self.val
    }
}

impl<T: Int> Deref for RangedInt<T> {
    type Target = T;

    fn deref(&self) -> &T {
        &self.val
    }
}


#[cfg(test)]
mod test_mod {
    use super::RangedInt;

    #[test]
    #[should_fail]
    fn test_new_value_greater_than_max() {
        let c = RangedInt::new(10i8, (-5, 5));
    }

    #[test]
    #[should_fail]
    fn test_new_value_lesser_than_min() {
        let c = RangedInt::new(-10i8, (-5, 5));
    }

    #[test]
    #[should_fail]
    fn test_min_bound_greater_than_max() {
        let c = RangedInt::new(0i8, (5, -5));
    }

    #[test]
    fn test_identical_bounds() {
        let mut c = RangedInt::new(0i8, (0, 0));
        assert_eq!(*c, 0);
        c.add(100);
        assert_eq!(*c, 0);
        c.add(-120);
        assert_eq!(*c, 0);
    }

    #[test]
    #[should_fail]
    fn set_value_over_max() {
        let mut c = RangedInt::new(0i8, (-5, 5));
        c.set(10);
    }

    #[test]
    #[should_fail]
    fn set_value_under_min() {
        let mut c = RangedInt::new(0i8, (-5, 5));
        c.set(-10);
    }


    #[test]
    fn set_value() {
        let mut c = RangedInt::new(0i8, (-5, 5));
        assert_eq!(*c, 0);

        c.set(0); assert_eq!(*c, 0);

        c.set(-5); assert_eq!(*c, -5);
        c.set(-4); assert_eq!(*c, -4);
        c.set(-3); assert_eq!(*c, -3);
        c.set(-2); assert_eq!(*c, -2);
        c.set(-1); assert_eq!(*c, -1);

        c.set(0); assert_eq!(*c, 0);

        c.set(1); assert_eq!(*c, 1);
        c.set(2); assert_eq!(*c, 2);
        c.set(3); assert_eq!(*c, 3);
        c.set(4); assert_eq!(*c, 4);
        c.set(5); assert_eq!(*c, 5);
    }

    #[test]
    fn test_add_overflow() {
        let mut c = RangedInt::new(0i8, (-5, 5));
        assert_eq!(*c, 0);

        c.add(1);
        assert_eq!(*c, 1);
        c.add(4);
        assert_eq!(*c, 5);
        c.add(1);
        assert_eq!(*c, 5);
        c.add(127);
        assert_eq!(*c, 5);
        c.add(-1);
        assert_eq!(*c, 4);

        c.set(0);
        assert_eq!(*c, 0);

        c.add(-3);
        assert_eq!(*c, -3);
        c.add(-3);
        assert_eq!(*c, -5);
        c.add(-128);
        assert_eq!(*c, -5);

        c.add(127);
        assert_eq!(*c, 5);
        c.add(127);
        assert_eq!(*c, 5);
        c.add(127);
        assert_eq!(*c, 5);
        c.add(127);
        assert_eq!(*c, 5);
        c.add(-128);
        assert_eq!(*c, -5);
        c.add(-128);
        assert_eq!(*c, -5);
        c.add(-128);
        assert_eq!(*c, -5);
    }
}
