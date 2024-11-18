//! Interpolated animations
use std::time::Duration;

use easing::{linear, round_trip, Easing};

pub mod easing;

mod manager;
#[doc(inline)]
pub use manager::Animations;
pub use manager::{AnimationMut, AnimationRef, Id};

struct Keyframe {
    easing: Easing,
    scheduled: Duration,
    requested: Option<Duration>,
}

/// An animation with keyframes
///
/// Keyframes are evaluated first to last
///
/// This interpolates time, with easing functions in the range of `0.0..=1.0`
#[derive(Default)]
pub struct Animation {
    keyframes: Vec<Keyframe>,
    scheduled: f32,
    current: f32,
    repeat: bool,
    round_trip: bool,
    oneshot: bool,
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
            oneshot: false,
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

    /// Should this animation be discarded once its done?
    pub fn oneshot(mut self, oneshot: bool) -> Self {
        self.oneshot = oneshot;
        self
    }

    /// Add this keyframe.
    ///
    /// The duration of this will come from an even distribution of keyframes that don't specify their time
    pub fn with(mut self, easing: Easing) -> Self {
        self.keyframes.push(Keyframe {
            easing,
            scheduled: Duration::ZERO,
            requested: None,
        });
        self
    }

    /// Add this keyframe with a specific duration
    pub fn with_time(mut self, easing: Easing, duration: Duration) -> Self {
        self.keyframes.push(Keyframe {
            easing,
            scheduled: Duration::ZERO,
            requested: Some(duration),
        });
        self
    }

    /// Recreate the keyframe schedule over a `total_time`
    ///
    /// If no keyframes were added, an error is returned.
    ///
    /// If `total_time` is less than the provided key frames, an error is returned.
    pub fn reschedule(&mut self, total_time: impl Into<Duration>) -> Result<(), &'static str> {
        if self.keyframes.is_empty() {
            return Err("No keyfounds were provided");
        }

        let total_time = total_time.into().as_secs_f32();
        let total_duration = self.keyframes.iter().fold(0.0, |a, frame| {
            a + frame.requested.as_ref().map_or(0.0, Duration::as_secs_f32)
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

        for frame in &mut self.keyframes {
            if frame.requested.is_none() {
                let dt = 1.0 / count as f32 * self.scheduled * scale;
                frame.scheduled = Duration::from_secs_f32(dt)
            }
        }

        Ok(())
    }

    /// Create the keyframe schedule over a `total_time`
    ///
    /// If no keyframes were added, an error is returned.
    ///
    /// If `total_time` is less than the provided key frames, an error is returned.
    pub fn schedule(mut self, total_time: impl Into<Duration>) -> Result<Self, &'static str> {
        self.reschedule(total_time).map(|_| self)
    }

    /// Returns whether the animation is done and should be discarded
    pub fn is_done(&self) -> bool {
        self.current > self.scheduled && self.oneshot
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
        for frame in &self.keyframes {
            elapsed += frame.scheduled;
            if self.current <= elapsed.as_secs_f32() {
                let time = (self.current - (elapsed - frame.scheduled).as_secs_f32())
                    / frame.scheduled.as_secs_f32();
                let apply = if self.round_trip { round_trip } else { linear };
                self.position = apply((frame.easing)(time));
                break;
            }
        }

        self.position
    }
}
