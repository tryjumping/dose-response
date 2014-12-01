use std::num::Int;


struct RangedInt<T: Int> {
    val: T,
    _min: T,
    _max: T,
}

impl<T: Int> RangedInt<T> {
    pub fn new(value: T, min: T, max: T) -> RangedInt<T> {
        assert!(min <= max);
        RangedInt {
            val: value,
            _min: min,
            _max: max,
        }
    }

    pub fn set(&mut self, n: T) {
        assert!(n >= self._min) && (n <= self._max);
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

impl<T: Int> Deref<T> for RangedInt<T> {
    fn deref(&self) -> &T {
        &self.val
    }
}


#[cfg(test)]
mod test_mod {
    use super::RangedInt;

    #[test]
    fn test_use_overflow() {
        let mut c = RangedInt::new(0i8, -5, 5);
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
