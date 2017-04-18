use std::cmp;

use player::{Bonus, Mind};
use ranged_int::{InclusiveRange, Ranged};


pub const ANXIETIES_PER_WILL: InclusiveRange = InclusiveRange(0, 7);

pub const WILL: InclusiveRange = InclusiveRange(0, 5);
pub const WITHDRAWAL: InclusiveRange = InclusiveRange(0, 15);
pub const SOBER: InclusiveRange = InclusiveRange(0, 20);
pub const HIGH: InclusiveRange = InclusiveRange(0, 80);
pub const SOBRIETY_COUNTER: InclusiveRange = InclusiveRange(0, 100);
pub const PANIC_TURNS: InclusiveRange = InclusiveRange(0, 10);
pub const STUN_TURNS: InclusiveRange = InclusiveRange(0, 10);


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


pub fn mind_take_turn(mind: Mind) -> Mind {
    use self::Mind::*;
    match mind {
        Withdrawal(value) => Withdrawal(value - 1),
        Sober(value) => {
            let new_value = value - 1;
            if new_value.is_min() {
                Withdrawal(Ranged::new_max(WITHDRAWAL))
            } else {
                Sober(new_value)
            }
        }
        High(value) => {
            let new_value = value - 1;
            if new_value.is_min() {
                Withdrawal(Ranged::new_max(WITHDRAWAL))
            } else {
                High(new_value)
            }
        }
    }
}


/// Update the `Mind` when eating food or being hit by the Hunger
/// monster.
pub fn process_hunger(mind: Mind, amount: i32) -> Mind {
    match mind {
        Mind::Withdrawal(val) => {
            if (*val + amount) > val.max() {
                let new_val = Ranged::new_min(SOBER);
                Mind::Sober(new_val + (val.max() - *val + amount))
            } else {
                Mind::Withdrawal(val + amount)
            }
        }

        Mind::Sober(val) => {
            if (*val + amount) >= val.min() {
                Mind::Sober(val + amount)
            } else {
                let new_val = Ranged::new_max(WITHDRAWAL);
                let amount = val.min() - *val + amount;
                Mind::Withdrawal(new_val + amount)
            }
        }

        Mind::High(val) => {
            // NOTE: Food and Hunger are the only users of
            // the attribute modifier so far.
            //
            // For hunger, we want it to go down even
            // while High but it should not increase the
            // intoxication value.
            let amount = cmp::min(0, amount);
            Mind::High(val + amount)
        }
    }
}


pub fn intoxicate(mind: Mind, tolerance: i32, expected_increment: i32) -> Mind {
    let increment = cmp::max(10, expected_increment - tolerance);

    // If we're high, the increment adds to the current intoxication
    // value, otherwise we go to high directly, ignoring any
    // withdrawn/sober states.
    match mind {
        Mind::Withdrawal(_) |
        Mind::Sober(_) => Mind::High(Ranged::new(increment, HIGH)),
        Mind::High(val) => Mind::High(val + increment),
    }
}


pub fn mind_bonus(mind: Mind) -> Option<Bonus> {
    match mind {
        Mind::High(val) if *val == val.max() - 1 => Some(Bonus::UncoverMap),
        Mind::High(val) if *val == val.max() - 2 => {
            Some(Bonus::SeeMonstersAndItems)
        }
        _ => None,
    }
}
