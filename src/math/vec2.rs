use crate::math::{pos2, Pos2};

// TODO make this into Size and provide a different Vec2
#[derive(Copy, Clone, Default, PartialEq, Eq, Hash)]
pub struct Vec2 {
    pub x: i32,
    pub y: i32,
}

impl std::fmt::Debug for Vec2 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Vec2({x}, {y})", x = self.x, y = self.y)
    }
}

impl Vec2 {
    pub const ZERO: Self = Self::splat(0);
    pub const MIN: Self = Self::splat(i32::MIN);
    pub const MAX: Self = Self::splat(i32::MAX);

    pub const MIN_X: Self = Self::new(i32::MIN, 0);
    pub const MIN_Y: Self = Self::new(0, i32::MIN);
    pub const MAX_X: Self = Self::new(i32::MAX, 0);
    pub const MAX_Y: Self = Self::new(0, i32::MAX);

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

impl std::ops::Add for Vec2 {
    type Output = Self;
    fn add(self, rhs: Vec2) -> Self::Output {
        vec2(self.x + rhs.x, self.y + rhs.y)
    }
}

impl std::ops::Sub for Vec2 {
    type Output = Self;
    fn sub(self, rhs: Vec2) -> Self::Output {
        vec2(self.x - rhs.x, self.y - rhs.y)
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

impl std::ops::Neg for Vec2 {
    type Output = Self;
    fn neg(self) -> Self::Output {
        Self::new(-self.x, -self.y)
    }
}

impl From<i32> for Vec2 {
    fn from(value: i32) -> Self {
        Self::splat(value)
    }
}

impl From<(i32, i32)> for Vec2 {
    fn from((x, y): (i32, i32)) -> Self {
        Self::new(x, y)
    }
}

impl From<[i32; 2]> for Vec2 {
    fn from([x, y]: [i32; 2]) -> Self {
        Self::new(x, y)
    }
}

impl From<f32> for Vec2 {
    fn from(value: f32) -> Self {
        Self::splat(value as i32)
    }
}
impl From<(f32, f32)> for Vec2 {
    fn from((x, y): (f32, f32)) -> Self {
        Self::new(x as i32, y as i32)
    }
}

impl From<[f32; 2]> for Vec2 {
    fn from([x, y]: [f32; 2]) -> Self {
        Self::new(x as i32, y as i32)
    }
}

impl From<Vec2> for (f32, f32) {
    fn from(value: Vec2) -> Self {
        (value.x as f32, value.y as f32)
    }
}

impl From<Vec2> for (i32, i32) {
    fn from(value: Vec2) -> Self {
        (value.x, value.y)
    }
}

impl From<Vec2> for Pos2 {
    fn from(value: Vec2) -> Self {
        pos2(value.x, value.y)
    }
}
