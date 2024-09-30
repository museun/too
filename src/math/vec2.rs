use crate::math::{pos2, Pos2};

#[derive(Copy, Clone, Default, PartialEq, Eq, Hash)]
pub struct Vec2 {
    pub x: i32,
    pub y: i32,
}

impl std::fmt::Debug for Vec2 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("Vec2").field(&self.x).field(&self.y).finish()
    }
}

impl Vec2 {
    pub const ZERO: Self = Self::splat(0);

    pub const fn new(x: i32, y: i32) -> Self {
        vec2(x, y)
    }

    pub const fn splat(d: i32) -> Self {
        vec2(d, d)
    }

    pub fn min(&self, other: Self) -> Self {
        vec2(self.x.min(other.x), self.y.min(other.y))
    }

    pub fn max(&self, other: Self) -> Self {
        vec2(self.x.max(other.x), self.y.max(other.y))
    }

    pub fn clamp(&self, min: Self, max: Self) -> Self {
        vec2(self.x.clamp(min.x, max.x), self.y.clamp(min.y, max.y))
    }

    pub const fn to_pos2(&self) -> Pos2 {
        pos2(self.x, self.y)
    }
}

pub const fn vec2(x: i32, y: i32) -> Vec2 {
    Vec2 { x, y }
}

impl std::ops::Add<i32> for Vec2 {
    type Output = Self;
    fn add(self, rhs: i32) -> Self::Output {
        vec2(self.x + rhs, self.y + rhs)
    }
}

impl std::ops::Sub<i32> for Vec2 {
    type Output = Self;
    fn sub(self, rhs: i32) -> Self::Output {
        vec2(self.x - rhs, self.y - rhs)
    }
}

impl std::ops::Mul<i32> for Vec2 {
    type Output = Self;
    fn mul(self, rhs: i32) -> Self::Output {
        vec2(self.x * rhs, self.y * rhs)
    }
}

impl std::ops::Div<i32> for Vec2 {
    type Output = Self;
    fn div(self, rhs: i32) -> Self::Output {
        vec2(self.x / rhs, self.y / rhs)
    }
}
