use core::f32;

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Margin {
    pub left: f32,
    pub top: f32,
    pub right: f32,
    pub bottom: f32,
}

impl Margin {
    pub const ZERO: Self = Self::same(0.0);
    pub const ONE: Self = Self::same(1.0);

    pub const fn new(left: f32, top: f32, right: f32, bottom: f32) -> Self {
        Self {
            left,
            top,
            right,
            bottom,
        }
    }

    pub const fn symmetric(x: f32, y: f32) -> Self {
        Self {
            left: x,
            top: y,
            right: x,
            bottom: y,
        }
    }

    pub const fn same(margin: f32) -> Self {
        Self::symmetric(margin, margin)
    }

    pub fn sum(&self) -> Size {
        Size::new(self.left + self.right, self.top + self.bottom)
    }

    pub const fn left_top(&self) -> Vector {
        Vector::new(self.left, self.top)
    }

    pub const fn right_bottom(&self) -> Vector {
        Vector::new(self.right, self.bottom)
    }

    pub fn expand_rect(&self, rect: impl Into<Rectf>) -> Rectf {
        let rect = rect.into();
        Rectf::new(rect.min - self.left_top(), rect.max + self.right_bottom())
    }

    pub fn shrink_rect(&self, rect: impl Into<Rectf>) -> Rectf {
        let rect = rect.into();
        Rectf::new(rect.min + self.left_top(), rect.max - self.right_bottom())
    }
}

impl From<[u16; 4]> for Margin {
    fn from([left, top, right, bottom]: [u16; 4]) -> Self {
        Self::new(left as f32, top as f32, right as f32, bottom as f32)
    }
}
impl From<(u16, u16)> for Margin {
    fn from((x, y): (u16, u16)) -> Self {
        Self::symmetric(x as f32, y as f32)
    }
}
impl From<u16> for Margin {
    fn from(value: u16) -> Self {
        Self::same(value as f32)
    }
}

impl From<[f32; 4]> for Margin {
    fn from([left, top, right, bottom]: [f32; 4]) -> Self {
        Self::new(left, top, right, bottom)
    }
}
impl From<(f32, f32)> for Margin {
    fn from((x, y): (f32, f32)) -> Self {
        Self::symmetric(x, y)
    }
}
impl From<f32> for Margin {
    fn from(value: f32) -> Self {
        Self::same(value)
    }
}

impl From<Size> for Margin {
    fn from(value: Size) -> Self {
        Self::symmetric(value.width, value.height)
    }
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Rectf {
    pub min: Point,
    pub max: Point,
}

impl Rectf {
    pub const ZERO: Self = Self::new(Point::ZERO, Point::ZERO);

    pub const fn new(min: Point, max: Point) -> Self {
        Self { min, max }
    }

    pub fn min_size(min: Point, size: Size) -> Self {
        Self {
            min,
            max: min + size,
        }
    }

    pub fn max_size(max: Point, size: Size) -> Self {
        Self {
            min: max - size,
            max,
        }
    }

    pub fn center_size(center: Point, size: Size) -> Self {
        Self {
            min: center - size * 0.5,
            max: center + size * 0.5,
        }
    }

    pub fn size(self) -> Size {
        Size::from(self.max - self.min)
    }

    pub fn area(self) -> f32 {
        self.width() * self.height()
    }

    pub fn width(self) -> f32 {
        self.max.x - self.min.x
    }

    pub fn height(self) -> f32 {
        self.max.y - self.min.y
    }

    pub fn round(self) -> Self {
        Self::new(self.min.round(), self.max.round())
    }

    pub fn clamp(self, other: impl Into<Self>) -> Self {
        let other = other.into();
        Self::new(
            self.min.clamp(other.min, other.max),
            self.max.clamp(other.min, other.max),
        )
    }

    pub fn offset(self) -> Vector {
        self.min.to_vector()
    }

    pub fn expand(self, d: f32) -> Self {
        self.expand2(Size::splat(d))
    }

    pub fn expand2(self, size: impl Into<Size>) -> Self {
        let size = size.into();
        Self::new(self.min + size, self.max + size)
    }

    pub fn shrink(self, d: f32) -> Self {
        self.shrink2(Size::splat(d))
    }

    pub fn shrink2(self, size: impl Into<Size>) -> Self {
        self.expand2(-size.into())
    }

    pub fn contains(self, point: Point) -> bool {
        let x = point.x >= self.min.x && point.x < self.max.x;
        let y = point.y >= self.min.y && point.y < self.max.y;
        x && y
    }

    pub fn clamp_point(self, point: Point) -> Point {
        let x = point.x.max(self.min.x).min(self.max.x);
        let y = point.y.max(self.min.y).min(self.max.y);
        Point::new(x, y)
    }

    pub fn include(self, point: Point) -> Self {
        Self::new(
            Point::new(f32::min(self.min.x, point.x), f32::min(self.max.y, point.y)),
            Point::new(f32::max(self.max.x, point.x), f32::max(self.max.y, point.y)),
        )
    }

    pub const fn left(self) -> f32 {
        self.min.x
    }

    pub const fn top(self) -> f32 {
        self.min.y
    }

    pub const fn right(self) -> f32 {
        self.max.x
    }

    pub const fn bottom(self) -> f32 {
        self.max.y
    }

    pub fn left_top(self) -> Point {
        Point::new(self.left(), self.top())
    }

    pub fn center_top(self) -> Point {
        Point::new(self.center().x, self.top())
    }

    pub fn right_top(self) -> Point {
        Point::new(self.right(), self.top())
    }

    pub fn left_center(self) -> Point {
        Point::new(self.left(), self.center().y)
    }

    pub fn center(self) -> Point {
        self.min + self.size() * 0.5
    }

    pub fn right_center(self) -> Point {
        Point::new(self.right(), self.center().y)
    }

    pub fn left_bottom(&self) -> Point {
        Point::new(self.left(), self.bottom())
    }

    pub fn center_bottom(self) -> Point {
        Point::new(self.center().x, self.bottom())
    }

    pub fn right_bottom(self) -> Point {
        Point::new(self.right(), self.bottom())
    }
}

impl From<too::math::Rect> for Rectf {
    fn from(value: too::math::Rect) -> Self {
        Self::new(
            Point::new(value.min.x as f32, value.min.y as f32),
            Point::new(value.max.x as f32, value.max.y as f32),
        )
    }
}

impl From<Rectf> for too::math::Rect {
    fn from(value: Rectf) -> Self {
        let rect = value.round();
        Self::from_min_max(
            too::math::pos2(rect.min.x as _, rect.min.y as _),
            too::math::pos2(rect.max.x as _, rect.max.y as _),
        )
    }
}

impl From<Size> for Rectf {
    fn from(value: Size) -> Self {
        Self::new(Point::ZERO, value.into())
    }
}

impl From<[f32; 4]> for Rectf {
    fn from([x1, y1, x2, y2]: [f32; 4]) -> Self {
        Self::new(Point::new(x1, y1), Point::new(x2, y2))
    }
}

impl From<(f32, f32)> for Rectf {
    fn from((w, h): (f32, f32)) -> Self {
        Self::min_size(Point::ZERO, Size::new(w, h))
    }
}

impl std::ops::Add<too::math::Vec2> for Rectf {
    type Output = Self;
    fn add(self, rhs: too::math::Vec2) -> Self::Output {
        self + Vector::from(rhs)
    }
}

impl std::ops::Add<too::math::Pos2> for Rectf {
    type Output = Self;
    fn add(self, rhs: too::math::Pos2) -> Self::Output {
        self + Vector::from(rhs.to_vec2())
    }
}

impl std::ops::Add<Vector> for Rectf {
    type Output = Self;
    fn add(self, rhs: Vector) -> Self::Output {
        Self::new(self.min + rhs, self.max + rhs)
    }
}

impl std::ops::Add<Size> for Rectf {
    type Output = Self;
    fn add(self, rhs: Size) -> Self::Output {
        Self::new(self.min, self.max + rhs)
    }
}

impl std::ops::Sub<too::math::Vec2> for Rectf {
    type Output = Self;
    fn sub(self, rhs: too::math::Vec2) -> Self::Output {
        self - Vector::from(rhs)
    }
}

impl std::ops::Sub<too::math::Pos2> for Rectf {
    type Output = Self;
    fn sub(self, rhs: too::math::Pos2) -> Self::Output {
        self - Vector::from(rhs.to_vec2())
    }
}

impl std::ops::Sub<Vector> for Rectf {
    type Output = Self;
    fn sub(self, rhs: Vector) -> Self::Output {
        Self::new(self.min - rhs, self.max - rhs)
    }
}

impl std::ops::Sub<Size> for Rectf {
    type Output = Self;
    fn sub(self, rhs: Size) -> Self::Output {
        Self::new(self.min, self.max - rhs)
    }
}

impl std::ops::AddAssign<too::math::Vec2> for Rectf {
    fn add_assign(&mut self, rhs: too::math::Vec2) {
        *self += Vector::from(rhs)
    }
}

impl std::ops::AddAssign<too::math::Pos2> for Rectf {
    fn add_assign(&mut self, rhs: too::math::Pos2) {
        *self += Vector::from(rhs.to_vec2())
    }
}

impl std::ops::AddAssign<Vector> for Rectf {
    fn add_assign(&mut self, rhs: Vector) {
        *self = *self + rhs
    }
}

impl std::ops::AddAssign<Size> for Rectf {
    fn add_assign(&mut self, rhs: Size) {
        *self = *self + rhs
    }
}

impl std::ops::SubAssign<too::math::Vec2> for Rectf {
    fn sub_assign(&mut self, rhs: too::math::Vec2) {
        *self -= Vector::from(rhs)
    }
}

impl std::ops::SubAssign<too::math::Pos2> for Rectf {
    fn sub_assign(&mut self, rhs: too::math::Pos2) {
        *self -= Vector::from(rhs.to_vec2())
    }
}

impl std::ops::SubAssign<Vector> for Rectf {
    fn sub_assign(&mut self, rhs: Vector) {
        *self = *self - rhs
    }
}

impl std::ops::SubAssign<Size> for Rectf {
    fn sub_assign(&mut self, rhs: Size) {
        *self = *self - rhs
    }
}

#[derive(Copy, Clone, PartialEq)]
pub struct Point {
    pub x: f32,
    pub y: f32,
}

impl std::fmt::Debug for Point {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_map()
            .entry(&crate::debug_fmt::str("x"), &self.x)
            .entry(&crate::debug_fmt::str("y"), &self.y)
            .finish()
    }
}

impl Default for Point {
    fn default() -> Self {
        Self::ZERO
    }
}

impl Point {
    pub const ZERO: Self = Self::new(0.0, 0.0);
    pub const ONE: Self = Self::new(1.0, 1.0);

    pub const X: Self = Self::new(1.0, 0.0);
    pub const Y: Self = Self::new(0.0, 1.0);

    pub const NEG_X: Self = Self::new(-1.0, 0.0);
    pub const NEG_Y: Self = Self::new(0.0, -1.0);

    pub const INFINITY: Self = Self::new(f32::INFINITY, f32::INFINITY);

    pub const fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }

    pub const fn splat(value: f32) -> Self {
        Self::new(value, value)
    }

    pub fn min(self, other: Self) -> Self {
        Self::new(self.x.min(other.x), self.y.min(other.y))
    }

    pub fn max(self, other: Self) -> Self {
        Self::new(self.x.max(other.x), self.y.max(other.y))
    }

    pub fn clamp(self, min: Self, max: Self) -> Self {
        Self::new(self.x.clamp(min.x, max.x), self.y.clamp(min.y, max.y))
    }

    pub fn floor(self) -> Self {
        Self::new(self.x.floor(), self.y.floor())
    }

    pub fn ceil(self) -> Self {
        Self::new(self.x.ceil(), self.y.ceil())
    }

    pub fn round(self) -> Self {
        Self::new(self.x.round(), self.y.round())
    }

    pub fn fract(self) -> Self {
        Self::new(self.x.fract(), self.y.fract())
    }

    pub fn is_finite(self) -> bool {
        self.x.is_finite() && self.y.is_finite()
    }

    pub fn is_infinite(self) -> bool {
        self.x.is_infinite() || self.y.is_infinite()
    }

    pub fn is_nan(self) -> bool {
        self.x.is_nan() || self.y.is_nan()
    }

    pub fn distance(self, other: Self) -> f32 {
        Vector::length(other - self)
    }

    pub fn lerp(self, other: Self, t: f32) -> Self {
        self - (other - self) * t
    }

    pub const fn to_vector(self) -> Vector {
        Vector::new(self.x, self.y)
    }

    pub const fn to_size(self) -> Size {
        Size::new(self.x, self.y)
    }
}

impl From<(f32, f32)> for Point {
    fn from((x, y): (f32, f32)) -> Self {
        Self::new(x, y)
    }
}

impl From<[f32; 2]> for Point {
    fn from([x, y]: [f32; 2]) -> Self {
        Self::new(x, y)
    }
}

impl From<f32> for Point {
    fn from(value: f32) -> Self {
        Self::splat(value)
    }
}

impl From<Vector> for Point {
    fn from(value: Vector) -> Self {
        Self::new(value.x, value.y)
    }
}

impl From<Size> for Point {
    fn from(value: Size) -> Self {
        Self::new(value.width, value.height)
    }
}

impl std::ops::Neg for Point {
    type Output = Self;
    fn neg(self) -> Self::Output {
        Self::new(-self.x, -self.y)
    }
}

impl std::ops::Sub for Point {
    type Output = Vector;
    fn sub(self, rhs: Self) -> Self::Output {
        Vector::new(self.x - rhs.x, self.y - rhs.y)
    }
}

impl From<too::math::Pos2> for Point {
    fn from(value: too::math::Pos2) -> Self {
        Self::new(value.x as _, value.y as _)
    }
}

impl From<Point> for too::math::Pos2 {
    fn from(value: Point) -> Self {
        let value = value.round();
        too::math::pos2(value.x as i32, value.y as i32)
    }
}

macro_rules! point_ops {
    ($op:ident, $assign:ident, $func:ident, $assign_func:ident, $sigil:tt) => {
        impl std::ops::$op<Vector> for Point {
            type Output = Self;
            fn $func(self, rhs: Vector) -> Self::Output {
                Self::new(self.x $sigil rhs.x, self.y $sigil rhs.y)
            }
        }

        impl std::ops::$assign<Vector> for Point {
            fn $assign_func(&mut self, rhs: Vector)  {
                *self = *self $sigil rhs
            }
        }

        impl std::ops::$op<Size> for Point {
            type Output = Self;
            fn $func(self, rhs: Size) -> Self::Output {
                Self::new(self.x $sigil rhs.width, self.y $sigil rhs.height)
            }
        }

        impl std::ops::$assign<Size> for Point {
            fn $assign_func(&mut self, rhs: Size)  {
                *self = *self $sigil rhs
            }
        }

        impl std::ops::$op<f32> for Point {
            type Output = Self;
            fn $func(self, rhs: f32) -> Self::Output {
                Self::new(self.x $sigil rhs, self.y $sigil rhs)
            }
        }

        impl std::ops::$assign<f32> for Point {
            fn $assign_func(&mut self, rhs: f32)  {
                *self = *self $sigil rhs
            }
        }
    };
}

point_ops!(Add, AddAssign, add, add_assign, +);
point_ops!(Sub, SubAssign, sub, sub_assign, -);
point_ops!(Mul, MulAssign, mul, mul_assign, *);
point_ops!(Div, DivAssign, div, div_assign, /);
point_ops!(Rem, RemAssign, rem, rem_assign, %);

// why does this exist?
#[derive(Copy, Clone, PartialEq)]
pub struct Vector {
    pub x: f32,
    pub y: f32,
}

impl std::fmt::Debug for Vector {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_map()
            .entry(&crate::debug_fmt::str("x"), &self.x)
            .entry(&crate::debug_fmt::str("y"), &self.y)
            .finish()
    }
}

impl Vector {
    pub const ZERO: Self = Self::new(0.0, 0.0);
    pub const ONE: Self = Self::new(1.0, 1.0);

    pub const X: Self = Self::new(1.0, 0.0);
    pub const Y: Self = Self::new(0.0, 1.0);

    pub const NEG_X: Self = Self::new(-1.0, 0.0);
    pub const NEG_Y: Self = Self::new(0.0, -1.0);

    pub const fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }

    pub const fn splat(value: f32) -> Self {
        Self::new(value, value)
    }

    pub fn from_angle(angle: f32) -> Self {
        let (sin, cos) = angle.sin_cos();
        Self::new(cos, sin)
    }

    pub fn min(self, other: Self) -> Self {
        Self::new(self.x.min(other.x), self.y.min(other.y))
    }

    pub fn max(self, other: Self) -> Self {
        Self::new(self.x.max(other.x), self.y.max(other.y))
    }

    pub fn clamp(self, min: Self, max: Self) -> Self {
        Self::new(self.x.clamp(min.x, max.x), self.y.clamp(min.y, max.y))
    }

    pub fn floor(self) -> Self {
        Self::new(self.x.floor(), self.y.floor())
    }

    pub fn ceil(self) -> Self {
        Self::new(self.x.ceil(), self.y.ceil())
    }

    pub fn round(self) -> Self {
        Self::new(self.x.round(), self.y.round())
    }

    pub fn fract(self) -> Self {
        Self::new(self.x.fract(), self.y.fract())
    }

    pub fn is_finite(self) -> bool {
        self.x.is_finite() && self.y.is_finite()
    }

    pub fn is_infinite(self) -> bool {
        self.x.is_infinite() || self.y.is_infinite()
    }

    pub fn is_nan(self) -> bool {
        self.x.is_nan() || self.y.is_nan()
    }

    pub fn signum(self) -> Self {
        Self::new(self.x.signum(), self.y.signum())
    }

    #[doc(alias = "abs")]
    pub fn length(self) -> f32 {
        self.length_squared().sqrt()
    }

    pub fn length_squared(self) -> f32 {
        self.dot(self)
    }

    pub fn normalize(self) -> Self {
        let length = self.length();
        if length == 0.0 {
            Self::ZERO
        } else {
            self / length
        }
    }

    pub fn dot(self, other: Self) -> f32 {
        self.x * other.x + self.y * other.y
    }

    pub fn cross(self, other: Self) -> f32 {
        self.x * other.x - self.y * other.y
    }

    pub fn angle_between(self, other: Self) -> f32 {
        (self.dot(other) / f32::sqrt(self.length_squared() * other.length_squared())).acos()
    }

    pub fn angle(self) -> f32 {
        self.y.atan2(self.x)
    }

    pub const fn to_point(self) -> Point {
        Point::new(self.x, self.y)
    }

    pub const fn to_size(self) -> Size {
        Size::new(self.x, self.y)
    }
}

impl From<(f32, f32)> for Vector {
    fn from((x, y): (f32, f32)) -> Self {
        Self::new(x, y)
    }
}

impl From<[f32; 2]> for Vector {
    fn from([x, y]: [f32; 2]) -> Self {
        Self::new(x, y)
    }
}

impl From<f32> for Vector {
    fn from(value: f32) -> Self {
        Self::splat(value)
    }
}

impl From<Point> for Vector {
    fn from(value: Point) -> Self {
        Self::new(value.x, value.y)
    }
}

impl From<Size> for Vector {
    fn from(value: Size) -> Self {
        Self::new(value.width, value.height)
    }
}

impl From<too::math::Vec2> for Vector {
    fn from(value: too::math::Vec2) -> Self {
        Self::new(value.x as f32, value.y as f32)
    }
}

impl From<Vector> for too::math::Vec2 {
    fn from(value: Vector) -> Self {
        Self::new(value.x.round() as _, value.y.round() as _)
    }
}

impl std::ops::Neg for Vector {
    type Output = Self;
    fn neg(self) -> Self::Output {
        Self::new(-self.x, -self.y)
    }
}

macro_rules! vector_ops {
    ($op:ident, $assign:ident, $func:ident, $assign_func:ident, $sigil:tt) => {
        impl std::ops::$op<Point> for Vector {
            type Output = Self;
            fn $func(self, rhs: Point) -> Self::Output {
                Self::new(self.x $sigil rhs.x, self.y $sigil rhs.y)
            }
        }

        impl std::ops::$assign<Point> for Vector {
            fn $assign_func(&mut self, rhs: Point)  {
                *self = *self $sigil rhs
            }
        }

        impl std::ops::$op<Size> for Vector {
            type Output = Self;
            fn $func(self, rhs: Size) -> Self::Output {
                Self::new(self.x $sigil rhs.width, self.y $sigil rhs.height)
            }
        }

        impl std::ops::$assign<Size> for Vector {
            fn $assign_func(&mut self, rhs: Size)  {
                *self = *self $sigil rhs
            }
        }

        impl std::ops::$op<f32> for Vector {
            type Output = Self;
            fn $func(self, rhs: f32) -> Self::Output {
                Self::new(self.x $sigil rhs, self.y $sigil rhs)
            }
        }

        impl std::ops::$assign<f32> for Vector {
            fn $assign_func(&mut self, rhs: f32)  {
                *self = *self $sigil rhs
            }
        }
    };
}

vector_ops!(Add, AddAssign, add, add_assign, +);
vector_ops!(Sub, SubAssign, sub, sub_assign, -);
vector_ops!(Mul, MulAssign, mul, mul_assign, *);
vector_ops!(Div, DivAssign, div, div_assign, /);
vector_ops!(Rem, RemAssign, rem, rem_assign, %);

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Size {
    pub width: f32,
    pub height: f32,
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

    pub const fn to_point(self) -> Point {
        Point::new(self.width, self.height)
    }

    pub const fn to_vector(self) -> Size {
        Size::new(self.width, self.height)
    }
}

impl From<Size> for (f32, f32) {
    fn from(size: Size) -> Self {
        (size.width, size.height)
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

impl From<f32> for Size {
    fn from(value: f32) -> Self {
        Self::splat(value)
    }
}

impl From<Point> for Size {
    fn from(value: Point) -> Self {
        Self::new(value.x, value.y)
    }
}

impl From<Vector> for Size {
    fn from(value: Vector) -> Self {
        Self::new(value.x, value.y)
    }
}

impl From<Size> for too::math::Pos2 {
    fn from(value: Size) -> Self {
        let size = value.round();
        too::math::pos2(size.width.round() as _, size.height.round() as _)
    }
}

impl From<too::math::Vec2> for Size {
    fn from(value: too::math::Vec2) -> Self {
        Self::new(value.x as f32, value.y as f32)
    }
}

impl From<Size> for too::math::Vec2 {
    fn from(value: Size) -> Self {
        let size = value.round();
        too::math::vec2(size.width.round() as _, size.height.round() as _)
    }
}

impl std::ops::Neg for Size {
    type Output = Self;
    fn neg(self) -> Self::Output {
        Self::new(-self.width, -self.height)
    }
}

// TODO this is totally from the wrong crate
impl std::ops::Mul<too::layout::Align2> for Size {
    type Output = Self;
    fn mul(self, rhs: too::layout::Align2) -> Self::Output {
        fn factor(align: too::layout::Align) -> f32 {
            match align {
                too::layout::Align::Min => 0.0,
                too::layout::Align::Center => 0.5,
                too::layout::Align::Max => 1.0,
            }
        }
        Self::new(self.width * factor(rhs.x), self.height * factor(rhs.y))
    }
}

macro_rules! size_ops {
    ($op:ident, $assign:ident, $func:ident, $assign_func:ident, $sigil:tt) => {
        impl std::ops::$op for Size {
            type Output = Self;
            fn $func(self, rhs: Self) -> Self::Output {
                Self::new(self.width $sigil rhs.width, self.height $sigil rhs.height)
            }
        }

        impl std::ops::$assign for Size {
            fn $assign_func(&mut self, rhs: Self)  {
                *self = *self $sigil rhs
            }
        }


        impl std::ops::$op<Point> for Size {
            type Output = Self;
            fn $func(self, rhs: Point) -> Self::Output {
                Self::new(self.width $sigil rhs.x, self.height $sigil rhs.y)
            }
        }

        impl std::ops::$assign<Point> for Size {
            fn $assign_func(&mut self, rhs: Point)  {
                *self = *self $sigil rhs
            }
        }

        impl std::ops::$op<Vector> for Size {
            type Output = Self;
            fn $func(self, rhs: Vector) -> Self::Output {
                Self::new(self.width $sigil rhs.x, self.height $sigil rhs.y)
            }
        }

        impl std::ops::$assign<Vector> for Size {
            fn $assign_func(&mut self, rhs: Vector)  {
                *self = *self $sigil rhs
            }
        }

        impl std::ops::$op<f32> for Size {
            type Output = Self;
            fn $func(self, rhs: f32) -> Self::Output {
                Self::new(self.width $sigil rhs, self.height $sigil rhs)
            }
        }

        impl std::ops::$assign<f32> for Size {
            fn $assign_func(&mut self, rhs: f32)  {
                *self = *self $sigil rhs
            }
        }
    };
}

size_ops!(Add, AddAssign, add, add_assign, +);
size_ops!(Sub, SubAssign, sub, sub_assign, -);
size_ops!(Mul, MulAssign, mul, mul_assign, *);
size_ops!(Div, DivAssign, div, div_assign, /);
size_ops!(Rem, RemAssign, rem, rem_assign, %);

// TODO this goes into the layout module
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

    pub const fn from_size(size: Size) -> Self {
        Self::new(size, size)
    }

    pub fn shrink(self, size: Size) -> Self {
        let min = self.min - size;
        let max = self.max - size;
        Self::new(min.max(Size::ZERO), max.max(Size::ZERO))
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

impl From<Size> for Space {
    fn from(value: Size) -> Self {
        Self::new(value, value)
    }
}

impl std::ops::BitAnd for Space {
    type Output = Self;
    fn bitand(self, rhs: Self) -> Self::Output {
        self.constrain(rhs)
    }
}

impl std::ops::BitAndAssign for Space {
    fn bitand_assign(&mut self, rhs: Self) {
        *self = *self & rhs;
    }
}

impl std::ops::Add<Size> for Space {
    type Output = Self;
    fn add(self, rhs: Size) -> Self::Output {
        self.expand(rhs)
    }
}

impl std::ops::AddAssign<Size> for Space {
    fn add_assign(&mut self, rhs: Size) {
        *self = *self + rhs;
    }
}

impl std::ops::Sub<Size> for Space {
    type Output = Self;
    fn sub(self, rhs: Size) -> Self::Output {
        self.shrink(rhs)
    }
}

impl std::ops::SubAssign<Size> for Space {
    fn sub_assign(&mut self, rhs: Size) {
        *self = *self - rhs;
    }
}
