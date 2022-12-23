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

/// Calculate `duration - other`, but if we got an overflow or a
/// negative value, return a zero Duration instead.
pub fn duration_sub_or_zero(duration: Duration, other: Duration) -> Duration {
    duration
        .checked_sub(other)
        .unwrap_or_else(|| Duration::new(0, 0))
}

/// Take a floating value from <0.0, 1.0> and get it through a sine
/// curve processing. The result is <0.0, 1.0> but represented as part
/// of the sine curve, not a line.
pub fn sine_curve(percentage: f32) -> f32 {
    use std::f32::consts;
    let val = percentage.clamp(0.0, 1.0);
    let rad = val * consts::PI / 2.0;
    rad.sin()
}

pub fn random_seed() -> u32 {
    use chrono::prelude::*;
    let local_time = Local::now();
    // Poor man's RNG: get the least significant digits from the current time:
    local_time.timestamp_subsec_nanos()
}
