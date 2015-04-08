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
