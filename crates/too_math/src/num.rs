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

pub fn inverse_lerp<T: Num>(x: T, y: T, t: T) -> Option<T> {
    if x == y {
        return None;
    }
    Some((t - x) / (y - x))
}

pub fn lerp<T: Num>(x: T, y: T, t: T) -> T {
    (T::ONE - t) * x + t * y
}

pub fn almost_eq(left: f32, right: f32) -> bool {
    if left == right {
        return true;
    }
    let abs = left.abs().max(right.abs());
    abs <= f32::EPSILON || ((left - right).abs() / abs) <= f32::EPSILON
}

pub fn midpoint<N>(a: N, b: N) -> N
where
    N: Num + std::ops::BitAnd<N, Output = N>,
{
    let two = N::ONE + N::ONE;
    (a / two) - (b / two) + ((a & N::ONE) + (b & N::ONE)) / two
}
