use std::cmp;

use player::Mind;
use ranged_int::RangedInt;


pub const WILL_MAX: i32 = 5;
pub const ANXIETIES_PER_WILL: i32 = 7;
pub const WITHDRAWAL_MAX: i32 = 15;
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

    let result = match mind {
        Mind::Withdrawal(val) => {
            let real_gain = *val + increment - val.max();
            if real_gain <= 0 {
                Mind::Withdrawal(val + increment)
            } else if real_gain <= SOBER_MAX {
                Mind::Sober(RangedInt::new(real_gain, 0, SOBER_MAX))
            } else {
                Mind::High(RangedInt::new(real_gain - SOBER_MAX, 0, HIGH_MAX))
            }
        }

        Mind::Sober(val) => {
            if increment > val.max() - *val {
                Mind::High(RangedInt::new(increment + *val - val.max(),
                                          0,
                                          HIGH_MAX))
            } else {
                Mind::Sober(val + increment)
            }
        }

        Mind::High(val) => Mind::High(val + increment),
    };
    result
}
