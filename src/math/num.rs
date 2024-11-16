use std::ops::RangeInclusive;

pub trait Num:
    PartialOrd
    + std::fmt::Debug
    + std::ops::Add<Output = Self>
    + std::ops::Sub<Output = Self>
    + std::ops::Mul<Output = Self>
    + std::ops::Div<Output = Self>
    + Copy
{
    const ZERO: Self;
    const ONE: Self;
}

impl Num for i32 {
    const ZERO: Self = 0;
    const ONE: Self = 1;
}

impl Num for u16 {
    const ZERO: Self = 0;
    const ONE: Self = 1;
}

impl Num for f32 {
    const ZERO: Self = 0.0;
    const ONE: Self = 1.0;
}

// TODO use ULPS instead of the abs average approach + ULPS
pub fn almost_eq(left: f32, right: f32) -> bool {
    if left == right {
        return true;
    }
    let abs = left.abs().max(right.abs());
    abs <= f32::EPSILON || ((left - right).abs() / abs) <= f32::EPSILON
}

pub fn inverse_lerp<T: Num>(x: T, y: T, t: T) -> Option<T> {
    if x == y {
        return None;
    }
    Some((t - x) / (y - x))
}

pub fn lerp<T: Num>(x: T, y: T, t: T) -> T {
    (T::ONE - t) * x + t * y
}

/// inverse lerp
pub fn normalize(value: f32, range: RangeInclusive<f32>) -> f32 {
    let value = value.clamp(*range.start(), *range.end());
    (value - range.start()) / (range.end() - range.start())
}

/// lerp
pub fn denormalize(value: f32, range: RangeInclusive<f32>) -> f32 {
    let value = value.clamp(0.0, 1.0);
    value * (range.end() - range.start()) + range.start()
}

pub fn remap<N: Num>(val: N, from: RangeInclusive<N>, to: RangeInclusive<N>) -> N {
    let (x1, y1) = (*from.start(), *from.end());
    let (x2, y2) = (*to.start(), *to.end());
    // inverse lerp to map val to the from range
    // lerp the result with the to range to get the remapped value
    let t = lerp(x2, y2, (val - x1) / (y1 - x1));
    clamp_num(t, x2, y2)
}

fn clamp_num<N: Num>(value: N, min: N, max: N) -> N {
    if value < min {
        return min;
    }
    if value > max {
        return max;
    }
    value
}
