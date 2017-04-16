use std::cmp;

use player::Mind;
use ranged_int::RangedInt;


pub const WILL_MAX: i32 = 5;
pub const ANXIETIES_PER_WILL: i32 = 7;
pub const WITHDRAWAL_MAX: i32 = 15;
pub const HIGH_MIN: i32 = 0;
pub const HIGH_MAX: i32 = 80;
pub const SOBER_MAX: i32 = 20;


pub fn exploration_radius(mental_state: Mind) -> i32 {
    use player::Mind::*;
    match mental_state {
        Withdrawal(value) => if *value >= value.middle() { 5 } else { 4 },
        Sober(_) => 6,
        High(value) => if *value >= value.middle() { 8 } else { 7 },
    }
}


pub fn player_resist_radius(dose_irresistible_value: i32, will: i32) -> i32 {
    cmp::max(dose_irresistible_value + 1 - will, 0)
}


pub fn intoxicate(mind: Mind, tolerance: i32, expected_increment: i32) -> Mind {
    let increment = cmp::max(10, expected_increment - tolerance);

    // If we're high, the increment adds to the current intoxication
    // value, otherwise we go to high directly, ignoring any
    // withdrawn/sober states.
    match mind {
        Mind::Withdrawal(_) | Mind::Sober(_) => {
            Mind::High(RangedInt::new(increment, HIGH_MIN, HIGH_MAX))
        }
        Mind::High(val) => Mind::High(val + increment),
    }
}
