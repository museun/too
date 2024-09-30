use crate::math::{vec2, Vec2};

#[derive(Copy, Clone, Default, PartialEq, Eq, Hash)]
pub struct Pos2 {
    pub x: i32,
    pub y: i32,
}

impl std::fmt::Debug for Pos2 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("Pos2").field(&self.x).field(&self.y).finish()
    }
}

impl Pos2 {
    pub const ZERO: Self = Self::splat(0);

    pub const fn new(x: i32, y: i32) -> Self {
        pos2(x, y)
    }

    pub const fn splat(d: i32) -> Self {
        pos2(d, d)
    }

    pub fn min(&self, other: Self) -> Self {
        pos2(self.x.min(other.x), self.y.min(other.y))
    }

    pub fn max(&self, other: Self) -> Self {
        pos2(self.x.max(other.x), self.y.max(other.y))
    }

    pub fn clamp(&self, min: Self, max: Self) -> Self {
        pos2(self.x.clamp(min.x, max.x), self.y.clamp(min.y, max.y))
    }

    pub const fn length_sq(&self) -> i32 {
        self.x * self.x + self.y * self.y
    }

    pub const fn to_vec2(self) -> Vec2 {
        vec2(self.x, self.y)
    }
}

pub const fn pos2(x: i32, y: i32) -> Pos2 {
    Pos2 { x, y }
}

impl std::ops::Add for Pos2 {
    type Output = Pos2;
    fn add(self, rhs: Self) -> Self::Output {
        pos2(self.x + rhs.x, self.y + rhs.y)
    }
}

impl std::ops::AddAssign for Pos2 {
    fn add_assign(&mut self, rhs: Self) {
        *self = *self + rhs
    }
}

impl std::ops::Sub for Pos2 {
    type Output = Self;
    fn sub(self, rhs: Self) -> Self::Output {
        pos2(self.x - rhs.x, self.y - rhs.y)
    }
}

impl std::ops::Add<Vec2> for Pos2 {
    type Output = Self;
    fn add(self, rhs: Vec2) -> Self::Output {
        pos2(self.x + rhs.x, self.y + rhs.y)
    }
}

impl std::ops::Sub<Vec2> for Pos2 {
    type Output = Self;
    fn sub(self, rhs: Vec2) -> Self::Output {
        pos2(self.x - rhs.x, self.y - rhs.y)
    }
}
