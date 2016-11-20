use std::ops::Deref;


pub struct RangedInt {
    val: i32,
    _min: i32,
    _max: i32,
}

impl RangedInt {
    pub fn new(value: i32, (min, max): (i32, i32)) -> RangedInt {
        assert!(min <= max && value >= min && value <= max);
        RangedInt {
            val: value,
            _min: min,
            _max: max,
        }
    }

    pub fn set(&mut self, n: i32) {
        assert!((n >= self._min) && (n <= self._max));
        self.val = n;
    }

    pub fn add(&mut self, n: i32) -> i32 {
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

impl Deref for RangedInt {
    type Target = i32;

    fn deref(&self) -> &i32 {
        &self.val
    }
}
