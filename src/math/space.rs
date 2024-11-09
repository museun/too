use super::Size;

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Space {
    pub min: Size,
    pub max: Size,
}

impl Default for Space {
    fn default() -> Self {
        Self::UNBOUNDED
    }
}

impl Space {
    pub const ZERO: Self = Self {
        min: Size::ZERO,
        max: Size::ZERO,
    };

    pub const UNBOUNDED: Self = Self {
        min: Size::ZERO,
        max: Size::INFINITY,
    };

    pub const FILL: Self = Self {
        min: Size::INFINITY,
        max: Size::INFINITY,
    };

    pub const fn new(min: Size, max: Size) -> Self {
        Self { min, max }
    }

    pub fn tight(size: impl Into<Size>) -> Self {
        Self::from_size(size.into())
    }

    pub const fn from_size(size: Size) -> Self {
        Self::new(size, size)
    }

    pub fn shrink(self, size: Size) -> Self {
        let min = (self.min + size).max(Size::ZERO);
        let max = (self.max - size).max(Size::ZERO);
        Self::new(min, max)
    }

    pub fn expand(self, size: Size) -> Self {
        Self::new(self.min + size, self.max + size)
    }

    pub fn loosen(self) -> Self {
        Self::new(Size::ZERO, self.max)
    }

    pub fn loosen_width(mut self) -> Self {
        self.min.width = 0.0;
        self
    }

    pub fn loosen_height(mut self) -> Self {
        self.min.height = 0.0;
        self
    }

    pub fn is_finite(self) -> bool {
        self.min.is_finite() && self.max.is_finite()
    }

    pub fn is_infinite(self) -> bool {
        self.min.is_infinite() && self.max.is_infinite()
    }

    pub fn constrain(self, other: Self) -> Self {
        let min = self.min.max(other.min);
        let max = self.max.min(other.max);
        Self::new(min.min(max), max)
    }

    pub fn constrain_min(self, size: Size) -> Size {
        size.max(self.min)
    }

    pub fn fit(self, size: Size) -> Size {
        let width = if self.min.width.is_finite() {
            size.width.max(self.min.width)
        } else {
            size.width
        };
        let height = if self.min.height.is_finite() {
            size.height.max(self.min.height)
        } else {
            size.height
        };
        Size::new(width.min(self.max.width), height.min(self.max.height))
    }

    pub fn size(&self) -> Size {
        let mut size = Size::ZERO;

        if self.max.width.is_finite() {
            size.width = self.max.width
        } else if self.min.width.is_finite() {
            size.width = self.min.width
        }

        if self.max.height.is_finite() {
            size.height = self.max.height
        } else if self.min.height.is_finite() {
            size.height = self.min.height
        }

        size
    }
}

impl std::ops::Add<Size> for Space {
    type Output = Self;
    fn add(self, rhs: Size) -> Self::Output {
        let min = (self.min + rhs).max(Size::ZERO);
        let max = (self.max + rhs).max(Size::ZERO);
        Self::new(min, max)
    }
}

impl std::ops::Sub<Size> for Space {
    type Output = Self;
    fn sub(self, rhs: Size) -> Self::Output {
        let min = (self.min - rhs).max(Size::ZERO);
        let max = (self.max - rhs).max(Size::ZERO);
        Self::new(min, max)
    }
}
