//! Interpolated animations
use std::time::Duration;

use easing::{linear, round_trip, Easing};

pub mod easing;

mod manager;
pub use manager::{AnimationManager, AnimationMut, AnimationRef};

/// An animation with keyframes
///
/// Keyframes are evaluated first to last
///
/// This interpolates time, with easing functions in the range of `0.0..=1.0`
#[derive(Default)]
pub struct Animation {
    keyframes: Vec<(Easing, Option<Duration>)>,
    scheduled: f32,
    current: f32,
    repeat: bool,
    round_trip: bool,
    position: f32,
}

impl Animation {
    pub const fn new() -> Self {
        Self {
            keyframes: Vec::new(),
            scheduled: 0.0,
            current: 0.0,
            repeat: false,
            round_trip: false,
            position: 0.0,
        }
    }

    /// Should this animation repeat?
    pub fn repeat(mut self, repeat: bool) -> Self {
        self.repeat = repeat;
        self
    }

    /// Should the keyframes round trip?
    pub fn round_trip(mut self, round_trip: bool) -> Self {
        self.round_trip = round_trip;
        self
    }

    /// Add this keyframe.
    ///
    /// The duration of this will come from an even distribution of keyframes that don't specify their time
    pub fn with(mut self, easing: Easing) -> Self {
        self.keyframes.push((easing, None));
        self
    }

    /// Add this keyframe with a specific duration
    pub fn with_time(mut self, easing: Easing, duration: Duration) -> Self {
        self.keyframes.push((easing, duration.into()));
        self
    }

    /// Create the keyframe schedule over a `total_time`
    ///
    /// If no keyframes were added, an error is returned.
    ///
    /// If `total_time` is less than the provided key frames, an error is returned.
    pub fn schedule(mut self, total_time: impl Into<Duration>) -> Result<Self, &'static str> {
        if self.keyframes.is_empty() {
            return Err("No keyfounds were provided");
        }

        let total_time = total_time.into().as_secs_f32();
        let total_duration = self.keyframes.iter().fold(0.0, |a, (_, d)| {
            a + d.as_ref().map_or(0.0, Duration::as_secs_f32)
        });

        if total_duration > total_time {
            return Err("Total keyframe duration exceeds scheduled time");
        }

        self.scheduled = total_time;
        let count = self.keyframes.len();

        let scale = if total_duration > self.scheduled {
            self.scheduled / total_duration
        } else {
            1.0
        };

        for (_, dur) in &mut self.keyframes {
            if dur.is_none() {
                *dur = Some(Duration::from_secs_f32(
                    1.0 / count as f32 * self.scheduled * scale,
                ))
            }
        }

        Ok(self)
    }

    /// Reset the position of each keyframe (e.g. this animation is reset to zero)
    pub fn reset(&mut self) {
        self.position = 0.0;
        self.current = 0.0;
    }

    /// Update this animation with a delta time, returning the new value
    pub fn update(&mut self, dt: f32) -> f32 {
        self.current += dt;

        if self.current > self.scheduled {
            if !self.repeat {
                return self.position;
            }
            self.current %= self.scheduled
        }

        let mut elapsed = Duration::ZERO;
        for (func, dur) in &self.keyframes {
            let dur = dur.unwrap(); // we set it above
            elapsed += dur;
            if self.current <= elapsed.as_secs_f32() {
                let time = (self.current - (elapsed - dur).as_secs_f32()) / dur.as_secs_f32();
                let apply = if self.round_trip { round_trip } else { linear };
                self.position = apply(func(time));
                break;
            }
        }

        self.position
    }
}
