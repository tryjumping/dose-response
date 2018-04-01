use std::time::Duration;

#[cfg(not(feature = "web"))]
use rand;

/// The number of nanoseconds in a microsecond.
const NANOS_PER_MICRO: u32 = 1000;
/// The number of nanoseconds in a millisecond.
const NANOS_PER_MILLI: u32 = 1000_000;
/// The number of microseconds per second.
const MICROS_PER_SEC: u64 = 1000_000;
/// The number of milliseconds per second.
const MILLIS_PER_SEC: u64 = 1000;
/// The number of seconds in a minute.

pub fn num_milliseconds(duration: Duration) -> u64 {
    let secs_part = duration.as_secs() * MILLIS_PER_SEC;
    let nanos_part = duration.subsec_nanos() / NANOS_PER_MILLI;
    secs_part + nanos_part as u64
}

pub fn num_microseconds(duration: Duration) -> Option<u64> {
    if let Some(secs_part) = duration.as_secs().checked_mul(MICROS_PER_SEC) {
        let nanos_part = duration.subsec_nanos() / NANOS_PER_MICRO;
        return secs_part.checked_add(nanos_part as u64);
    }
    None
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


#[cfg(not(feature = "web"))]
pub fn random_seed() -> u32 {
    rand::random::<u32>()
}

#[cfg(feature = "web")]
pub fn random_seed() -> u32 {
    #[allow(unsafe_code)]
    // NOTE: this comes from `Math.random` and returns a float in the <0, 1> range:
    let random_float = unsafe { ::engine::wasm::random() };
    (random_float * ::std::u32::MAX as f32) as u32
}
