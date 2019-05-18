use crate::{
    item::{Item, Kind},
    monster::CompanionBonus,
    player::{Bonus, CauseOfDeath, Mind, Modifier, Player},
    point::Point,
    ranged_int::{InclusiveRange, Ranged},
};

use std::cmp;

use num_rational::{Ratio, Rational32};
use rand::Rng;

#[cfg(feature = "recording")]
pub const INITIAL_SAFE_RADIUS: i32 = 7;
#[cfg(not(feature = "recording"))]
pub const INITIAL_SAFE_RADIUS: i32 = 15;

#[cfg(feature = "recording")]
pub const INITIAL_EASY_RADIUS: i32 = 13;
#[cfg(not(feature = "recording"))]
pub const INITIAL_EASY_RADIUS: i32 = 25;

pub const NO_LETHAL_DOSE_RADIUS: i32 = 6;

pub const ANXIETIES_PER_WILL: InclusiveRange = InclusiveRange(0, 7);

pub const WILL: InclusiveRange = InclusiveRange(0, 6);

// The rate at which the Mind drops under normal circumstances
pub const MIND_DROP_PER_TURN: i32 = 1;

// NOTE: We use the MIND_DROP_PER_TURN multiple here. That way, unless
// it's modified, the number here contains the default pace in turns.
pub const WITHDRAWAL: InclusiveRange = InclusiveRange(0, 15);
pub const SOBER: InclusiveRange = InclusiveRange(0, 20);
pub const HIGH: InclusiveRange = InclusiveRange(0, 80);

pub const VICTORY_NPC_DISTANCE: InclusiveRange = InclusiveRange(80, 120);

pub const CHUNK_BASELINE_DENSITY: f32 = 0.39;
pub const CHUNK_DENSITY_VARIABILITY: (f32, f32) = (-0.30, 0.15);

#[cfg(feature = "recording")]
pub const DOSE_PREFAB: Item = Item {
    kind: Kind::Dose,
    irresistible: 2,
    modifier: Modifier::Intoxication {
        state_of_mind: 30,
        tolerance_increase: 4,
    },
};

#[cfg(not(feature = "recording"))]
pub const DOSE_PREFAB: Item = Item {
    kind: Kind::Dose,
    irresistible: 2,
    modifier: Modifier::Intoxication {
        state_of_mind: 70,
        tolerance_increase: 4,
    },
};

pub const STRONG_DOSE_PREFAB: Item = Item {
    kind: Kind::StrongDose,
    irresistible: 4,
    modifier: Modifier::Intoxication {
        state_of_mind: 220,
        tolerance_increase: 7,
    },
};

pub const CARDINAL_DOSE_PREFAB: Item = Item {
    kind: Kind::CardinalDose,
    irresistible: 3,
    modifier: Modifier::Intoxication {
        state_of_mind: 130,
        tolerance_increase: 6,
    },
};

pub const DIAGONAL_DOSE_PREFAB: Item = Item {
    kind: Kind::DiagonalDose,
    irresistible: 3,
    modifier: Modifier::Intoxication {
        state_of_mind: 130,
        tolerance_increase: 6,
    },
};

pub const FOOD_PREFAB: Item = Item {
    kind: Kind::Food,
    irresistible: 0,
    modifier: Modifier::Attribute {
        state_of_mind: 20,
        will: 0,
    },
};

// This how much a given dose can vary from the prefab's base value
pub const DOSE_MIND_VARIANCE: InclusiveRange = InclusiveRange(-5, 5);
pub const STRONG_DOSE_MIND_VARIANCE: InclusiveRange = InclusiveRange(-15, -15);
pub const CARDINAL_DOSE_MIND_VARIANCE: InclusiveRange = InclusiveRange(-10, 10);
pub const DIAGONAL_DOSE_MIND_VARIANCE: InclusiveRange = InclusiveRange(-10, 10);

pub const PLAYER_BASE_AP: i32 = 1;
pub const PLAYER_STARTING_WILL: i32 = 2;
pub const PANIC_TURNS: InclusiveRange = InclusiveRange(0, 10);
pub const STUN_TURNS: InclusiveRange = InclusiveRange(0, 10);

pub const CHASING_DISTANCE: i32 = 5;
pub const HOWLING_DISTANCE: i32 = 15;

pub const ESTRANGED_NPC_MAX_AP: i32 = 2;

pub const FRIENDLY_NPC_FREEZE_RADIUS: f32 = 3.0;

pub const PATHFINDING_MONSTER_LIMIT: i32 = 50;
pub const PATHFINDING_DOSE_RESIST_LIMIT: i32 = 50;
pub const PATHFINDING_VNPC_REACHABILITY_LIMIT: i32 = 500;

pub fn exploration_radius(mental_state: Mind) -> i32 {
    use crate::player::Mind::*;
    match mental_state {
        Withdrawal(value) => {
            if value.to_int() >= value.middle() {
                5
            } else {
                4
            }
        }
        Sober(_) => 6,
        High(value) => {
            if value.to_int() >= value.middle() {
                8
            } else {
                7
            }
        }
    }
}

pub fn player_resist_radius(dose_irresistible_value: i32, will: i32) -> i32 {
    cmp::max(dose_irresistible_value + 2 - will, 0)
}

pub fn mind_drop_per_turn(bonuses: &[CompanionBonus]) -> Rational32 {
    if bonuses.contains(&CompanionBonus::HalveExhaustion) {
        Ratio::new(MIND_DROP_PER_TURN, 2)
    } else {
        Ratio::from_integer(MIND_DROP_PER_TURN)
    }
}

pub fn mind_take_turn(mind: Mind, drop: Rational32) -> Mind {
    use self::Mind::*;
    match mind {
        Withdrawal(value) => Withdrawal(value - drop),
        Sober(value) => {
            let new_value = value - drop;
            if new_value.is_min() {
                Withdrawal(Ranged::new_max(WITHDRAWAL))
            } else {
                Sober(new_value)
            }
        }
        High(value) => {
            let new_value = value - drop;
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
            if (val.to_int() + amount) > val.max() {
                let new_val = Ranged::new_min(SOBER);
                Mind::Sober(new_val + (amount - (val.max() - val.to_int())))
            } else {
                Mind::Withdrawal(val + amount)
            }
        }

        Mind::Sober(val) => {
            if (val.to_int() + amount) >= val.min() {
                Mind::Sober(val + amount)
            } else {
                let new_val = Ranged::new_max(WITHDRAWAL);
                let amount = val.min() - val.to_int() + amount;
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
    let increment = cmp::max(15, expected_increment - tolerance);

    // If we're high, the increment adds to the current intoxication
    // value, otherwise we go to high directly, ignoring any
    // withdrawn/sober states.
    match mind {
        Mind::Withdrawal(_) | Mind::Sober(_) => Mind::High(Ranged::new(increment, HIGH)),
        Mind::High(val) => Mind::High(val + increment),
    }
}

pub fn mind_bonus(mind: Mind) -> Option<Bonus> {
    match mind {
        Mind::High(val) if val.to_int() == val.max() - 1 => Some(Bonus::UncoverMap),
        Mind::High(val) if val.to_int() == val.max() - 2 => Some(Bonus::SeeMonstersAndItems),
        _ => None,
    }
}

pub fn cause_of_death(player: &Player) -> Option<CauseOfDeath> {
    use self::CauseOfDeath::*;
    match player.mind {
        Mind::Withdrawal(val) if val.is_min() => return Some(Exhausted),
        Mind::High(val) if val.is_max() => return Some(Overdosed),
        _ => {}
    }

    if player.will.is_min() {
        return Some(LostWill);
    }

    if player.dead {
        return Some(Killed);
    }

    None
}

pub fn mind_fade_value(mind: Mind) -> f32 {
    use crate::player::Mind::*;
    match mind {
        Withdrawal(value) => value.percent() * 0.5 + 0.45,
        Sober(_) | High(_) => 1.0,
    }
}

pub fn victory_npc_position<R: Rng>(
    rng: &mut R,
    player_pos: Point,
    distance_range: InclusiveRange,
) -> Point {
    let distance = distance_range.random(rng) as f32;
    let direction_rad: f32 = rng.gen_range(0.0, 2.0 * ::std::f32::consts::PI);
    let offset = Point {
        // TODO: we may have to add `cos` to wasm
        x: (direction_rad.cos() * distance) as i32,
        y: (direction_rad.sin() * distance) as i32,
    };
    player_pos + offset
}

pub fn player_max_ap(bonuses: &[CompanionBonus]) -> i32 {
    if bonuses.contains(&CompanionBonus::ExtraActionPoint) {
        PLAYER_BASE_AP + 1
    } else {
        PLAYER_BASE_AP
    }
}
