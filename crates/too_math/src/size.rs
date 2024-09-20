use crate::almost_eq;

pub const fn size(x: f32, y: f32) -> Size {
    Size::new(x, y)
}

#[derive(Copy, Clone, Debug)]
pub struct Size {
    pub x: f32,
    pub y: f32,
}

impl PartialEq for Size {
    fn eq(&self, other: &Self) -> bool {
        almost_eq(self.x, other.x) && almost_eq(self.y, other.y)
    }
}

impl Size {
    pub const ZERO: Self = size(0.0, 0.0);
    pub const INFINITY: Self = size(f32::INFINITY, f32::INFINITY);

    pub const fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }

    pub const fn splat(d: f32) -> Self {
        Self::new(d, d)
    }

    pub fn is_finite(&self) -> bool {
        self.x.is_finite() && self.y.is_finite()
    }

    pub fn is_infinite(&self) -> bool {
        !self.is_finite()
    }

    pub fn min(&self, other: Self) -> Self {
        size(f32::min(self.x, other.x), f32::min(self.y, other.y))
    }

    pub fn max(&self, other: Self) -> Self {
        size(f32::max(self.x, other.x), f32::min(self.y, other.y))
    }

    pub fn clamp(&self, min: Self, max: Self) -> Self {
        size(
            f32::clamp(self.x, min.x, max.x),
            f32::clamp(self.y, min.y, max.y),
        )
    }
}

impl Default for Size {
    fn default() -> Self {
        Self::ZERO
    }
}

impl std::ops::Add for Size {
    type Output = Self;
    fn add(self, rhs: Self) -> Self::Output {
        size(self.x + rhs.x, self.y + rhs.y)
    }
}
impl std::ops::Sub for Size {
    type Output = Self;
    fn sub(self, rhs: Self) -> Self::Output {
        size(self.x - rhs.x, self.y - rhs.y)
    }
}

impl std::ops::AddAssign for Size {
    fn add_assign(&mut self, rhs: Self) {
        *self = *self + rhs
    }
}
impl std::ops::SubAssign for Size {
    fn sub_assign(&mut self, rhs: Self) {
        *self = *self - rhs
    }
}

impl std::ops::Mul for Size {
    type Output = Self;
    fn mul(self, rhs: Self) -> Self::Output {
        size(self.x * rhs.x, self.y * rhs.y)
    }
}
impl std::ops::Div for Size {
    type Output = Self;
    fn div(self, rhs: Self) -> Self::Output {
        size(self.x / rhs.x, self.y / rhs.y)
    }
}

impl std::ops::MulAssign for Size {
    fn mul_assign(&mut self, rhs: Self) {
        *self = *self * rhs
    }
}
impl std::ops::DivAssign for Size {
    fn div_assign(&mut self, rhs: Self) {
        *self = *self / rhs
    }
}
