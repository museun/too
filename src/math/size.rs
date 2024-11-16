use crate::layout::Align2;

use super::{Margin, Pos2, Vec2};

#[derive(Copy, Clone, Default, PartialEq)]
pub struct Size {
    pub width: f32,
    pub height: f32,
}

impl std::fmt::Debug for Size {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Size({}, {})", self.width, self.height)
    }
}

impl Size {
    pub const ZERO: Self = Self::new(0.0, 0.0);
    pub const INFINITY: Self = Self::new(f32::INFINITY, f32::INFINITY);
    pub const NEG_INFINITY: Self = Self::new(f32::NEG_INFINITY, f32::NEG_INFINITY);
    pub const FILL: Self = Self::INFINITY;

    pub const fn new(width: f32, height: f32) -> Self {
        Self { width, height }
    }

    pub const fn splat(value: f32) -> Self {
        Self::new(value, value)
    }

    pub fn min(&self, other: Self) -> Self {
        Self::new(self.width.min(other.width), self.height.min(other.height))
    }

    pub fn max(&self, other: Self) -> Self {
        Self::new(self.width.max(other.width), self.height.max(other.height))
    }

    pub fn min_element(&self) -> f32 {
        self.width.min(self.height)
    }

    pub fn max_element(&self) -> f32 {
        self.width.max(self.height)
    }

    pub fn clamp(&self, min: Self, max: Self) -> Self {
        Self::new(
            self.width.clamp(min.width, max.width),
            self.height.clamp(min.height, max.height),
        )
    }

    pub fn floor(&self) -> Self {
        Self::new(self.width.floor(), self.height.floor())
    }

    pub fn ceil(&self) -> Self {
        Self::new(self.width.ceil(), self.height.ceil())
    }

    pub fn round(&self) -> Self {
        Self::new(self.width.round(), self.height.round())
    }

    pub fn fract(&self) -> Self {
        Self::new(self.width.fract(), self.height.fract())
    }

    pub fn is_finite(&self) -> bool {
        self.width.is_finite() && self.height.is_finite()
    }

    pub fn is_infinite(&self) -> bool {
        self.width.is_infinite() || self.height.is_infinite()
    }

    pub fn is_nan(&self) -> bool {
        self.width.is_nan() || self.height.is_nan()
    }

    pub fn finite_or_zero(&self) -> Self {
        fn map(d: f32) -> f32 {
            d.is_finite().then_some(d).unwrap_or_default()
        }
        Self::new(map(self.width), map(self.height))
    }
}

impl From<Size> for Pos2 {
    fn from(value: Size) -> Self {
        Pos2::new(value.width.round() as i32, value.height.round() as i32)
    }
}

impl From<f32> for Size {
    fn from(value: f32) -> Self {
        Self::splat(value)
    }
}

impl From<(f32, f32)> for Size {
    fn from((width, height): (f32, f32)) -> Self {
        Self::new(width, height)
    }
}

impl From<[f32; 2]> for Size {
    fn from([width, height]: [f32; 2]) -> Self {
        Self::new(width, height)
    }
}

impl From<[i32; 2]> for Size {
    fn from([width, height]: [i32; 2]) -> Self {
        Self::new(width as f32, height as f32)
    }
}

impl From<Size> for (f32, f32) {
    fn from(value: Size) -> Self {
        (value.width, value.height)
    }
}

impl From<i32> for Size {
    fn from(value: i32) -> Self {
        Self::splat(value as f32)
    }
}

impl From<(i32, i32)> for Size {
    fn from((width, height): (i32, i32)) -> Self {
        Self::new(width as f32, height as f32)
    }
}

impl From<Size> for Vec2 {
    fn from(size: Size) -> Self {
        // TODO handle infinites somehow
        Self::new(size.width.round() as i32, size.height.round() as i32)
    }
}

impl From<Vec2> for Size {
    fn from(size: Vec2) -> Self {
        Self::new(size.x as f32, size.y as f32)
    }
}

impl std::ops::Div<Align2> for Size {
    type Output = Self;
    fn div(self, rhs: Align2) -> Self::Output {
        let (x, y) = rhs.factor();
        Self::new(self.width / x, self.height / y)
    }
}

impl std::ops::Mul<Align2> for Size {
    type Output = Self;
    fn mul(self, rhs: Align2) -> Self::Output {
        let (x, y) = rhs.factor();
        Self::new(self.width * x, self.height * y)
    }
}

impl std::ops::Mul for Size {
    type Output = Self;
    fn mul(self, rhs: Self) -> Self::Output {
        Self::new(self.width * rhs.width, self.height * rhs.height)
    }
}

impl std::ops::Add for Size {
    type Output = Self;
    fn add(self, rhs: Self) -> Self::Output {
        Self::new(self.width + rhs.width, self.height + rhs.height)
    }
}

impl std::ops::Sub for Size {
    type Output = Self;
    fn sub(self, rhs: Self) -> Self::Output {
        Self::new(self.width - rhs.width, self.height - rhs.height)
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

impl std::ops::Add<Margin> for Size {
    type Output = Self;
    fn add(self, rhs: Margin) -> Self::Output {
        self + rhs.sum()
    }
}

impl std::ops::Sub<Margin> for Size {
    type Output = Self;
    fn sub(self, rhs: Margin) -> Self::Output {
        self - rhs.sum()
    }
}

impl std::ops::Mul<f32> for Size {
    type Output = Self;
    fn mul(self, rhs: f32) -> Self::Output {
        Self::new(self.width * rhs, self.height * rhs)
    }
}

impl std::ops::Div<f32> for Size {
    type Output = Self;
    fn div(self, rhs: f32) -> Self::Output {
        Self::new(self.width / rhs, self.height / rhs)
    }
}
