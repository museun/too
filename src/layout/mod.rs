//! Layout helpers

/// A direction such as _Horizontal_ or _Vertical_
#[derive(Copy, Clone, Debug, Default, PartialEq, PartialOrd, Eq, Ord, Hash)]
pub enum Axis {
    #[default]
    Horizontal,
    Vertical,
}

impl Axis {
    pub fn main<T>(&self, value: impl Into<(T, T)>) -> T {
        let (x, y) = value.into();
        match self {
            Self::Horizontal => x,
            Self::Vertical => y,
        }
    }

    pub fn cross<T>(&self, value: impl Into<(T, T)>) -> T {
        let (x, y) = value.into();
        match self {
            Self::Horizontal => y,
            Self::Vertical => x,
        }
    }

    pub fn pack<T, R>(&self, main: T, cross: T) -> R
    where
        R: From<(T, T)>,
    {
        match self {
            Self::Horizontal => R::from((main, cross)),
            Self::Vertical => R::from((cross, main)),
        }
    }

    pub fn unpack<T>(&self, value: impl Into<(T, T)>) -> (T, T) {
        let (x, y) = value.into();
        match self {
            Self::Horizontal => (x, y),
            Self::Vertical => (y, x),
        }
    }
}

impl std::ops::Neg for Axis {
    type Output = Self;
    fn neg(self) -> Self::Output {
        match self {
            Self::Horizontal => Self::Vertical,
            Self::Vertical => Self::Horizontal,
        }
    }
}

mod anchor;
pub use anchor::{Anchor, Anchor2};

mod linear;
pub use linear::{LinearAllocator, LinearLayout};

mod align;
pub use align::{Align, Align2};
