use super::{Pos2, Rect, Size, Vec2};

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
        self.sum() * Size::new(0.5, 1.0)
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
