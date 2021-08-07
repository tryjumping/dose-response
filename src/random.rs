use crate::util;

use oorandom::Rand32;

use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Random {
    #[serde(with = "derive_serde::Rand32Serde")]
    rng: Rand32,
}

impl Random {
    pub fn new() -> Self {
        Self::from_seed(u64::from(util::random_seed()))
    }

    pub fn from_seed(seed: u64) -> Self {
        Self {
            rng: oorandom::Rand32::new(seed),
        }
    }

    pub fn rand_float(&mut self) -> f32 {
        self.rng.rand_float()
    }

    pub fn range_inclusive(&mut self, start: i32, end: i32) -> i32 {
        let (diff, s) = if start < 0 {
            (0 - start, 0)
        } else {
            (0, start)
        };
        let e = (end + diff) + 1;
        assert!(s >= 0);
        assert!(e >= 0);
        assert!(diff >= 0);
        let s = s as u32;
        let e = e as u32;
        let positive_val = self.rng.rand_range(s..e);
        positive_val as i32 - diff
    }

    pub fn choose_weighted<'a, T>(&mut self, options: &'a [(T, u32)]) -> Option<&'a T> {
        if options.is_empty() {
            return None;
        };

        let all_weights = options.iter().map(|o| o.1).sum();
        let mut choice = self.rng.rand_range(0..all_weights);
        for (value, weight) in options {
            if choice <= *weight {
                return Some(value);
            }
            choice -= *weight;
        }
        None
    }

    pub fn choose<'a, T>(&mut self, options: &'a [T]) -> Option<&'a T> {
        if options.is_empty() {
            None
        } else {
            let index = self.rng.rand_range(0_u32..options.len() as u32) as usize;
            options.get(index)
        }
    }

    pub fn choose_with_fallback<'a, T>(&mut self, options: &'a [T], fallback: &'a T) -> &'a T {
        self.choose(options).unwrap_or(fallback)
    }
}

// This is a module that handles all the Serde trait deriving, tucked
// away from the main reason code for the `random` module. Basically
// we need this because oorandom doesn't provide the Serde traits and
// its fields are private.
mod derive_serde {
    use super::*;

    fn rand32_get_state(r: &Rand32) -> u64 {
        r.state().0
    }

    fn rand32_get_inc(r: &Rand32) -> u64 {
        r.state().1
    }

    #[allow(missing_copy_implementations)]
    #[derive(Clone, Debug, Serialize, Deserialize)]
    #[serde(remote = "Rand32")]
    pub struct Rand32Serde {
        #[serde(getter = "rand32_get_state")]
        state: u64,
        #[serde(getter = "rand32_get_inc")]
        inc: u64,
    }

    impl From<Rand32Serde> for Rand32 {
        fn from(r: Rand32Serde) -> Rand32 {
            Rand32::from_state((r.state, r.inc))
        }
    }
}
