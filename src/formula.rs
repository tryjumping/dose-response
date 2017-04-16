use std::cmp;

use player;


pub fn exploration_radius(mental_state: player::Mind) -> i32 {
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
