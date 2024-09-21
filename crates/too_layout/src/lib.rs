#![cfg_attr(debug_assertions, allow(dead_code, unused_variables,))]
use too_math::{vec2, Vec2};

#[derive(Copy, Clone, Debug, Default, PartialEq, PartialOrd, Eq, Ord, Hash)]
pub enum Direction {
    #[default]
    Horizontal,
    Vertical,
}

impl Direction {
    pub const fn size(&self, main: i32, cross: i32) -> Vec2 {
        match self {
            Self::Horizontal => vec2(main, cross),
            Self::Vertical => vec2(cross, main),
        }
    }

    pub const fn main(&self, size: Vec2) -> i32 {
        match self {
            Self::Horizontal => size.x,
            Self::Vertical => size.y,
        }
    }

    pub const fn cross(&self, size: Vec2) -> i32 {
        match self {
            Self::Horizontal => size.y,
            Self::Vertical => size.x,
        }
    }
}

// main alignment
// main spacing
// main distribution (sizing?)

// cross alignment
// cross spacing
// cross distribution (sizing?)

// flex layout
// box layout

mod linear;
pub use linear::{LinearAllocator, LinearLayout};

// pub mod flexible;
