//! Layout helpers

use too_math::vec2;
pub use too_math::{Rect, Vec2};

/// A direction such as _Horizontal_ or _Vertical_
#[derive(Copy, Clone, Debug, Default, PartialEq, PartialOrd, Eq, Ord, Hash)]
pub enum Axis {
    #[default]
    Horizontal,
    Vertical,
}

impl Axis {
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

mod anchor;
pub use anchor::{Anchor, Anchor2};

mod linear;
pub use linear::{LinearAllocator, LinearLayout};

mod align;
pub use align::{Align, Align2};

mod size;
pub use size::{size, Size};

mod constraints;
pub use constraints::Constraints;

// TODO helper to do row / column allocations (returning just the cross offset)
