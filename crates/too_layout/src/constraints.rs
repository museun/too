use crate::Size;

pub struct Constraints {
    pub min: Size,
    pub max: Size,
}

impl Constraints {
    pub const fn new(min: Size, max: Size) -> Self {
        Self { min, max }
    }

    pub const fn loose(max: Size) -> Self {
        Self {
            min: Size::ZERO,
            max,
        }
    }

    pub const fn tight(value: Size) -> Self {
        Self {
            min: value,
            max: value,
        }
    }

    pub const fn none() -> Self {
        Self {
            min: Size::ZERO,
            max: Size::INFINITY,
        }
    }

    pub fn constrain_min(&self, base: impl Into<Size>) -> Size {
        let base = base.into();
        base.max(self.min)
    }

    pub fn constrain(&self, base: impl Into<Size>) -> Size {
        self.constrain_min(base).min(self.max)
    }

    pub fn constrain_width(&self, width: f32) -> f32 {
        width.max(self.min.x).min(self.max.x)
    }

    pub fn constrain_height(&self, height: f32) -> f32 {
        height.max(self.min.y).min(self.max.y)
    }

    pub fn is_loose(&self) -> bool {
        self.min == Size::ZERO
    }

    pub fn is_tight(&self) -> bool {
        self.min == self.max
    }

    pub fn clamp(&self, min: Self, max: Self) -> Self {
        Self::new(
            self.min.clamp(min.min, min.max),
            self.max.clamp(max.min, max.max),
        )
    }

    pub fn as_size(&self) -> Size {
        match (self.max.is_finite(), self.min.is_finite()) {
            (true, ..) => self.max,
            (.., true) => self.min,
            _ => Size::ZERO,
        }
    }
}
