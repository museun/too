//! Layout helpers

use too_math::vec2;
pub use too_math::{Rect, Vec2};

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

#[derive(Copy, Clone, Debug, PartialEq, PartialOrd, Eq, Ord, Hash)]
pub struct Anchor2 {
    pub x: Anchor,
    pub y: Anchor,
}

impl Anchor2 {
    pub const LEFT_TOP: Self = Self {
        x: Anchor::LEFT,
        y: Anchor::TOP,
    };
    pub const RIGHT_TOP: Self = Self {
        x: Anchor::RIGHT,
        y: Anchor::TOP,
    };
    pub const RIGHT_BOTTOM: Self = Self {
        x: Anchor::RIGHT,
        y: Anchor::BOTTOM,
    };
    pub const LEFT_BOTTOM: Self = Self {
        x: Anchor::LEFT,
        y: Anchor::BOTTOM,
    };
}

impl Anchor2 {
    const fn sign(self, axis: Axis) -> (i32, i32) {
        match axis {
            Axis::Horizontal => (self.x.sign(), self.y.sign()),
            Axis::Vertical => (self.y.sign(), self.x.sign()),
        }
    }
}

#[derive(Copy, Clone, Debug, PartialEq, PartialOrd, Eq, Ord, Hash)]
pub enum Anchor {
    Min,
    Max,
}

impl Anchor {
    pub const LEFT: Self = Self::Min;
    pub const RIGHT: Self = Self::Max;
    pub const TOP: Self = Self::Min;
    pub const BOTTOM: Self = Self::Max;
}

impl Anchor {
    const fn sign(self) -> i32 {
        match self {
            Self::Min => 1,
            Self::Max => -1,
        }
    }

    const fn offset(self) -> i32 {
        match self {
            Self::Min => 0,
            Self::Max => 1,
        }
    }

    const fn exceeds_bounds(self, pos: i32, max: i32) -> bool {
        match self {
            Self::Min => pos > max,
            Self::Max => pos < max,
        }
    }
}

mod linear;
pub use linear::{LinearAllocator, LinearLayout};
