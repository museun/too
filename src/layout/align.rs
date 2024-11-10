/// Alignment on an axis
#[derive(Copy, Clone, Debug, Default, PartialEq, PartialOrd, Eq, Ord, Hash)]
pub enum Align {
    #[default]
    Min,
    Center,
    Max,
}

impl Align {
    pub const fn factor(&self) -> f32 {
        match self {
            Self::Min => 0.0,
            Self::Center => 0.5,
            Self::Max => 1.0,
        }
    }

    pub const fn align(&self, available: f32, size: f32) -> f32 {
        (available - size) * self.factor()
    }
}

impl Align {
    pub const LEFT: Self = Self::Min;
    pub const TOP: Self = Self::Min;
    pub const CENTER: Self = Self::Center;
    pub const RIGHT: Self = Self::Max;
    pub const BOTTOM: Self = Self::Max;

    pub const START: Self = Self::Min;
    pub const MIDDLE: Self = Self::Center;
    pub const END: Self = Self::Max;
}

/// Two dimensional alignment
#[derive(Copy, Clone, Debug, Default, PartialEq, PartialOrd, Eq, Ord, Hash)]
pub struct Align2 {
    /// Horizontal alignment
    pub x: Align,
    /// Vertical alignment
    pub y: Align,
}

impl Align2 {
    pub const fn factor(&self) -> (f32, f32) {
        (self.x.factor(), self.y.factor())
    }
}

impl Align2 {
    pub const LEFT_TOP: Self = Self {
        x: Align::LEFT,
        y: Align::TOP,
    };

    pub const CENTER_TOP: Self = Self {
        x: Align::CENTER,
        y: Align::TOP,
    };

    pub const RIGHT_TOP: Self = Self {
        x: Align::RIGHT,
        y: Align::TOP,
    };

    pub const LEFT_CENTER: Self = Self {
        x: Align::LEFT,
        y: Align::CENTER,
    };

    pub const CENTER_CENTER: Self = Self {
        x: Align::CENTER,
        y: Align::CENTER,
    };

    pub const RIGHT_CENTER: Self = Self {
        x: Align::RIGHT,
        y: Align::CENTER,
    };

    pub const LEFT_BOTTOM: Self = Self {
        x: Align::LEFT,
        y: Align::BOTTOM,
    };

    pub const CENTER_BOTTOM: Self = Self {
        x: Align::CENTER,
        y: Align::BOTTOM,
    };

    pub const RIGHT_BOTTOM: Self = Self {
        x: Align::RIGHT,
        y: Align::BOTTOM,
    };
}
