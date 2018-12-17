#![allow(dead_code)]

use std::time::Duration;

/// The number of nanoseconds in a microsecond.
const NANOS_PER_MICRO: u32 = 1000;
/// The number of nanoseconds in a millisecond.
const NANOS_PER_MILLI: u32 = 1_000_000;
/// The number of microseconds per second.
const MICROS_PER_SEC: u64 = 1_000_000;
/// The number of milliseconds per second.
const MILLIS_PER_SEC: u64 = 1000;
/// The number of seconds in a minute.

pub fn num_milliseconds(duration: Duration) -> u64 {
    let secs_part = duration.as_secs() * MILLIS_PER_SEC;
    let millis_part = duration.subsec_millis();
    secs_part + u64::from(millis_part)
}

pub fn num_microseconds(duration: Duration) -> Option<u64> {
    if let Some(secs_part) = duration.as_secs().checked_mul(MICROS_PER_SEC) {
        let micros_part = duration.subsec_micros();
        return secs_part.checked_add(u64::from(micros_part));
    }
    None
}

/// Calculate `duration - other`, but if we got an overflow or a
/// negative value, return a zero Duration instead.
pub fn duration_sub_or_zero(duration: Duration, other: Duration) -> Duration {
    duration.checked_sub(other).unwrap_or(Duration::new(0, 0))
}

/// If `val` is outside the `min` / `max` limits, set it to the edge value.
pub fn clamp(min: i32, val: i32, max: i32) -> i32 {
    if val < min {
        min
    } else if val > max {
        max
    } else {
        val
    }
}

/// If `val` is outside the `min` / `max` limits, set it to the edge value.
pub fn clampf(min: f32, val: f32, max: f32) -> f32 {
    if val < min {
        min
    } else if val > max {
        max
    } else {
        val
    }
}

/// Take a floating value from <0.0, 1.0> and get it through a sine
/// curve processing. The result is <0.0, 1.0> but represented as part
/// of the sine curve, not a line.
pub fn sine_curve(percentage: f32) -> f32 {
    use std::f32::consts;
    let val = clampf(0.0, percentage, 1.0);
    let rad = val * consts::PI / 2.0;
    rad.sin()
}

#[cfg(not(feature = "web"))]
pub fn random_seed() -> u32 {
    rand::random::<u32>()
}

#[cfg(feature = "web")]
pub fn random_seed() -> u32 {
    #[allow(unsafe_code)]
    // NOTE: this comes from `Math.random` and returns a float in the <0, 1> range:
    let random_float = unsafe { crate::engine::wasm::random() };
    (random_float * std::u32::MAX as f32) as u32
}
