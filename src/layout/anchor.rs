use super::Axis;

/// A two dimensional anchor
#[derive(Copy, Clone, Debug, PartialEq, PartialOrd, Eq, Ord, Hash)]
pub struct Anchor2 {
    /// Horizontal anchor
    pub x: Anchor,
    /// Vertical anchor
    pub y: Anchor,
}

impl Default for Anchor2 {
    fn default() -> Self {
        Self::LEFT_TOP
    }
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
    pub(crate) const fn sign(self, axis: Axis) -> (i32, i32) {
        match axis {
            Axis::Horizontal => (self.x.sign(), self.y.sign()),
            Axis::Vertical => (self.y.sign(), self.x.sign()),
        }
    }
}

/// Anchor on an axis
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
    pub(crate) const fn sign(self) -> i32 {
        match self {
            Self::Min => 1,
            Self::Max => -1,
        }
    }

    pub(crate) const fn offset(self) -> i32 {
        match self {
            Self::Min => 0,
            Self::Max => 1,
        }
    }

    pub(crate) const fn exceeds_bounds(self, pos: i32, max: i32) -> bool {
        match self {
            Self::Min => pos > max,
            Self::Max => pos < max,
        }
    }
}
