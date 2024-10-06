use crate::{
    layout::Align2,
    math::{Pos2, Rect, Vec2},
};

#[derive(Copy, Clone, Debug, PartialEq, PartialOrd)]
pub enum Flex {
    Tight(f32),
    Loose(f32),
}

impl Flex {
    pub const fn factor(&self) -> f32 {
        match self {
            Self::Tight(factor) | Self::Loose(factor) => *factor,
        }
    }

    pub const fn has_flex(&self) -> bool {
        self.factor() != 0.0
    }

    pub const fn is_expand(&self) -> bool {
        matches!(self, Self::Tight(..))
    }
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Margin {
    pub left: i32,
    pub top: i32,
    pub right: i32,
    pub bottom: i32,
}

impl Default for Margin {
    fn default() -> Self {
        Self::ZERO
    }
}

impl Margin {
    pub const ZERO: Self = Self::same(0);
    pub const ONE: Self = Self::same(1);

    pub const fn new(left: i32, top: i32, right: i32, bottom: i32) -> Self {
        Self {
            left,
            top,
            right,
            bottom,
        }
    }

    pub const fn symmetric(x: i32, y: i32) -> Self {
        Self {
            left: x,
            top: y,
            right: x,
            bottom: y,
        }
    }

    pub const fn same(margin: i32) -> Self {
        Self::symmetric(margin, margin)
    }

    pub fn sum(&self) -> Size {
        Size::new(
            self.left as f32 + self.right as f32,
            self.top as f32 + self.bottom as f32,
        )
    }

    pub fn size(&self) -> Size {
        self.sum() / 2.0
    }

    pub const fn left_top(&self) -> Pos2 {
        Pos2::new(self.left, self.top)
    }

    pub const fn right_bottom(&self) -> Pos2 {
        Pos2::new(self.right, self.bottom)
    }

    pub fn expand_rect(&self, rect: impl Into<Rect>) -> Rect {
        let rect = rect.into();
        Rect::from_min_max(rect.min - self.left_top(), rect.max + self.right_bottom())
    }

    pub fn shrink_rect(&self, rect: impl Into<Rect>) -> Rect {
        let rect = rect.into();
        Rect::from_min_max(rect.min + self.left_top(), rect.max - self.right_bottom())
    }

    // pub fn expand_space(&self, space: Space) -> (Pos2, Space) {
    //     (self.left_top(), space - self.sum())
    // }
}

impl From<i32> for Margin {
    fn from(value: i32) -> Self {
        Self::same(value)
    }
}

impl From<(i32, i32)> for Margin {
    fn from((x, y): (i32, i32)) -> Self {
        Self::symmetric(x, y)
    }
}

impl From<(i32, i32, i32, i32)> for Margin {
    fn from((left, top, right, bottom): (i32, i32, i32, i32)) -> Self {
        Self::new(left, top, right, bottom)
    }
}

impl From<[i32; 2]> for Margin {
    fn from([x, y]: [i32; 2]) -> Self {
        Self::symmetric(x, y)
    }
}

impl From<[i32; 4]> for Margin {
    fn from([left, top, right, bottom]: [i32; 4]) -> Self {
        Self::new(left, top, right, bottom)
    }
}

impl From<Margin> for Vec2 {
    fn from(value: Margin) -> Self {
        let Size { width, height } = value.size();
        Vec2::new(width.round() as i32, height.round() as i32)
    }
}

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
        match () {
            _ if self.max.is_finite() => self.max,
            _ if self.min.is_finite() => self.min,
            _ => Size::ZERO,
        }
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
        Self {
            width: self.width / x,
            height: self.height / y,
        }
    }
}

impl std::ops::Mul<Align2> for Size {
    type Output = Self;
    fn mul(self, rhs: Align2) -> Self::Output {
        let (x, y) = rhs.factor();
        Self {
            width: self.width * x,
            height: self.height * y,
        }
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
