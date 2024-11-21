use crate::math::{lerp, pos2, vec2, Pos2, Vec2};

#[derive(Copy, Clone, Default, PartialEq, Eq, Hash)]
pub struct Rect {
    pub min: Pos2,
    pub max: Pos2,
}

impl std::fmt::Debug for Rect {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Rect")
            .field("x", &self.min.x)
            .field("y", &self.min.y)
            .field("w", &self.width())
            .field("h", &self.height())
            .finish()
    }
}

impl Rect {
    pub const ZERO: Self = rect(Vec2::ZERO);

    pub const fn from_min_max(min: Pos2, max: Pos2) -> Self {
        Self { min, max }
    }

    pub const fn from_min_size(min: Pos2, size: Vec2) -> Self {
        Self {
            min,
            max: pos2(
                min.x.saturating_add_unsigned(size.x as u32),
                min.y.saturating_add_unsigned(size.y as u32),
            ),
        }
    }

    pub fn from_center_size(center: Pos2, size: Vec2) -> Self {
        Self {
            min: center - (size / 2),
            max: center + (size / 2),
        }
    }

    pub const fn area(&self) -> i32 {
        self.width() * self.height()
    }

    pub fn is_empty(&self) -> bool {
        self.size() == Vec2::ZERO
    }

    pub fn size(&self) -> Vec2 {
        (self.max - self.min).to_vec2()
    }

    pub fn clip(&self, size: Vec2) -> Self {
        Self::from_min_size(self.min, size.min(self.size()))
    }

    pub fn clamp(&self, pos: Pos2) -> Pos2 {
        pos.clamp(self.min, self.max)
    }

    pub fn clamp_rect(&self, other: Self) -> Self {
        let min = other.min.max(self.min).min(pos2(
            self.right().saturating_sub(other.width()),
            self.bottom().saturating_sub(other.height()),
        ));
        Self::from_min_size(min, other.size())
    }

    pub fn set_size(&mut self, size: impl Into<Vec2>) {
        *self = Self::from_min_size(self.min, size.into())
    }

    pub fn distance_to_point(&self, pos: Pos2) -> i32 {
        (self.distance_sq_to_point(pos) as f32).sqrt() as _
    }

    pub fn distance_sq_to_point(&self, pos: Pos2) -> i32 {
        const fn distance(min: i32, max: i32, t: i32) -> i32 {
            match () {
                _ if min > t => min - t,
                _ if t > max => t - max,
                _ => 0,
            }
        }

        let dx = distance(self.min.x, self.max.x, pos.x);
        let dy = distance(self.min.y, self.max.y, pos.y);
        pos2(dx, dy).length_sq()
    }

    pub const fn contains(&self, pos: Pos2) -> bool {
        self.min.x <= pos.x && pos.x < self.max.x && self.min.y <= pos.y && pos.y < self.max.y
    }

    pub const fn contains_inclusive(&self, pos: Pos2) -> bool {
        self.min.x <= pos.x && pos.x <= self.max.x && self.min.y <= pos.y && pos.y <= self.max.y
    }

    pub const fn contains_rect(&self, other: Self) -> bool {
        self.contains(other.min) && self.contains(other.max)
    }

    pub const fn contains_rect_inclusive(&self, other: Self) -> bool {
        self.contains_inclusive(other.min) && self.contains_inclusive(other.max)
    }

    pub const fn partial_contains_rect(&self, other: Self) -> bool {
        self.min.y <= other.max.y || self.min.x <= other.max.x && self.max.x >= other.min.y
    }

    pub fn shrink(&self, d: i32) -> Self {
        self.shrink2(Vec2::splat(d))
    }

    pub fn shrink2(&self, d: Vec2) -> Self {
        Self::from_min_max(self.min + d, self.max - d)
    }

    pub fn expand(&self, d: i32) -> Self {
        self.expand2(Vec2::splat(d))
    }

    pub fn expand2(&self, d: Vec2) -> Self {
        Self::from_min_max(self.min - d, self.max + d)
    }

    #[track_caller]
    pub fn translate(&self, vec: Vec2) -> Self {
        Self::from_min_size(self.min + vec, self.size())
    }

    pub fn with_size(&self, vec: Vec2) -> Self {
        Self::from_min_size(self.min, self.size() + vec)
    }

    pub fn intersection(&self, other: Self) -> Self {
        Self::from_min_max(self.min.max(other.min), self.max.min(other.max))
    }

    pub const fn intersects(&self, other: Self) -> bool {
        self.min.x <= other.max.x
            && other.min.x <= self.max.x
            && self.min.y <= other.min.y
            && other.max.y <= self.max.y
    }

    pub const fn partial_intersects(&self, other: &Self) -> bool {
        const fn contains_inclusive(left: &Rect, pos: Pos2) -> bool {
            left.min.x <= pos.x && pos.x <= left.max.x || left.min.y <= pos.y && pos.y <= left.max.y
        }
        contains_inclusive(self, other.min)
    }

    pub fn union(&self, other: Self) -> Self {
        Self::from_min_max(self.min.min(other.min), self.max.max(other.max))
    }

    pub const fn width(&self) -> i32 {
        self.max.x.saturating_sub(self.min.x)
    }

    pub const fn height(&self) -> i32 {
        self.max.y.saturating_sub(self.min.y)
    }

    pub const fn left(&self) -> i32 {
        self.min.x
    }

    pub const fn right(&self) -> i32 {
        self.max.x
    }

    pub const fn top(&self) -> i32 {
        self.min.y
    }

    pub const fn bottom(&self) -> i32 {
        self.max.y
    }

    pub fn center(&self) -> Pos2 {
        pos2(
            (self.max.x - self.min.x) / 2,
            (self.max.y - self.min.y) / 2,
            // TODO rounding
            // midpoint(self.min.x, self.max.x),
            // midpoint(self.min.y, self.max.y),
        )
    }

    pub const fn right_top(&self) -> Pos2 {
        pos2(self.right(), self.top())
    }

    pub const fn right_bottom(&self) -> Pos2 {
        pos2(self.right(), self.bottom())
    }

    pub const fn left_top(&self) -> Pos2 {
        pos2(self.left(), self.top())
    }

    pub const fn left_bottom(&self) -> Pos2 {
        pos2(self.left(), self.bottom())
    }

    // TODO spacing
    pub fn split_horizontal_n<const N: usize>(self) -> [Self; N] {
        let mut out = [Self::ZERO; N];
        let p = (self.width() as usize).div_ceil(N) as i32;
        let mut cursor = self.left_top();

        for temp in &mut out {
            *temp = Self::from_min_size(cursor, vec2(p, self.bottom()));
            cursor.x = temp.right() + 1;
        }

        let last = &mut out[N - 1];
        *last = Self::from_min_max(last.min, self.max);

        out
    }

    // TODO spacing
    pub fn split_vertical_n<const N: usize>(self) -> [Self; N] {
        let mut out = [Self::ZERO; N];
        let p = (self.height() as usize).div_ceil(N) as i32;
        let mut cursor = self.left_top();

        for temp in &mut out {
            *temp = Self::from_min_size(cursor, vec2(self.width(), p));
            cursor.y = temp.bottom() + 1;
        }

        let last = &mut out[N - 1];
        *last = Self::from_min_max(last.min, self.max);

        out
    }

    pub fn split_horizontal(self, spacing: i32, ratio: f32) -> (Self, Self) {
        let p = lerp(self.min.x as f32, self.max.x as f32, ratio) as i32;
        let left = Self::from_min_max(self.min, pos2(p - spacing, self.max.y));
        let right = Self::from_min_max(pos2(p, self.min.y), self.max);
        (left, right)
    }

    pub fn split_vertical(self, spacing: i32, ratio: f32) -> (Self, Self) {
        let p = lerp(self.min.y as f32, self.max.y as f32, ratio) as i32;
        let left = Self::from_min_max(self.min, pos2(self.max.x, p - spacing));
        let right = Self::from_min_max(pos2(self.min.x, p), self.max);
        (left, right)
    }

    pub fn corners(&self) -> [Pos2; 4] {
        [
            self.left_top(),
            self.right_top(),
            self.right_bottom(),
            self.left_bottom(),
        ]
    }
}

pub const fn rect(size: Vec2) -> Rect {
    Rect {
        min: Pos2::ZERO,
        max: pos2(size.x, size.y),
    }
}

impl std::ops::Add<Pos2> for Rect {
    type Output = Self;
    fn add(mut self, rhs: Pos2) -> Self::Output {
        self.min += rhs;
        self.max += rhs;
        self
    }
}

impl std::ops::Sub<Pos2> for Rect {
    type Output = Self;
    fn sub(mut self, rhs: Pos2) -> Self::Output {
        self.min -= rhs;
        self.max -= rhs;
        self
    }
}
