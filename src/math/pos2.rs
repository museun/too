use crate::math::{vec2, Vec2};

// TODO rename this to point
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
    type Output = Self;
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

impl std::ops::SubAssign for Pos2 {
    fn sub_assign(&mut self, rhs: Self) {
        *self = *self - rhs
    }
}

impl std::ops::Add<i32> for Pos2 {
    type Output = Self;
    fn add(self, rhs: i32) -> Self::Output {
        pos2(self.x + rhs, self.y + rhs)
    }
}

impl std::ops::AddAssign<i32> for Pos2 {
    fn add_assign(&mut self, rhs: i32) {
        *self = *self + rhs
    }
}

impl std::ops::Sub<i32> for Pos2 {
    type Output = Self;
    fn sub(self, rhs: i32) -> Self::Output {
        pos2(self.x - rhs, self.y - rhs)
    }
}

impl std::ops::SubAssign<i32> for Pos2 {
    fn sub_assign(&mut self, rhs: i32) {
        *self = *self - rhs
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

impl std::ops::AddAssign<Vec2> for Pos2 {
    fn add_assign(&mut self, rhs: Vec2) {
        *self = *self + rhs
    }
}

impl std::ops::SubAssign<Vec2> for Pos2 {
    fn sub_assign(&mut self, rhs: Vec2) {
        *self = *self - rhs
    }
}

impl std::ops::Neg for Pos2 {
    type Output = Self;
    fn neg(self) -> Self::Output {
        Self::new(-self.x, -self.y)
    }
}

impl std::ops::Mul<i32> for Pos2 {
    type Output = Self;
    fn mul(self, rhs: i32) -> Self::Output {
        Self::new(self.x * rhs, self.y * rhs)
    }
}

impl From<i32> for Pos2 {
    fn from(d: i32) -> Self {
        Self::splat(d)
    }
}

impl From<(i32, i32)> for Pos2 {
    fn from((x, y): (i32, i32)) -> Self {
        Self::new(x, y)
    }
}

impl From<[i32; 2]> for Pos2 {
    fn from([x, y]: [i32; 2]) -> Self {
        Self::new(x, y)
    }
}

impl From<f32> for Pos2 {
    fn from(d: f32) -> Self {
        Self::splat(d.round() as i32)
    }
}

impl From<(f32, f32)> for Pos2 {
    fn from((x, y): (f32, f32)) -> Self {
        let (x, y) = (x.round() as i32, y.round() as i32);
        Self::new(x, y)
    }
}

impl From<[f32; 2]> for Pos2 {
    fn from([x, y]: [f32; 2]) -> Self {
        let (x, y) = (x.round() as i32, y.round() as i32);
        Self::new(x, y)
    }
}

impl From<Pos2> for (i32, i32) {
    fn from(pos: Pos2) -> Self {
        (pos.x, pos.y)
    }
}

impl From<Pos2> for (f32, f32) {
    fn from(pos: Pos2) -> Self {
        (pos.x as f32, pos.y as f32)
    }
}
