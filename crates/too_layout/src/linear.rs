use too_math::{Pos2, Rect, Vec2};

use crate::Direction;

pub struct LinearAllocator {
    linear: LinearLayout,
    cursor: Pos2,
    rect: Rect,
    max: Vec2,
}

impl LinearAllocator {
    pub fn allocate(&mut self, size: Vec2) -> Option<Rect> {
        match self.linear.direction {
            Direction::Horizontal => self.horizontal(size),
            Direction::Vertical => self.vertical(size),
        }
    }

    pub fn spacing(&self) -> Vec2 {
        self.linear.spacing
    }

    pub fn start(&self) -> Pos2 {
        self.rect.left_top()
    }

    pub fn max_size(&self) -> Vec2 {
        self.rect.size()
    }

    pub fn remaining(&self) -> i32 {
        match self.linear.direction {
            Direction::Horizontal => self.rect.right().saturating_sub(self.cursor.x),
            Direction::Vertical => self.rect.bottom().saturating_sub(self.cursor.y),
        }
    }

    pub fn cursor(&self) -> Pos2 {
        self.cursor
    }

    fn horizontal(&mut self, size: Vec2) -> Option<Rect> {
        if self.cursor.x + size.x > self.rect.right() {
            if !self.linear.wrap {
                return None;
            }

            self.cursor.y += self.linear.spacing.y + size.y.max(self.max.y);
            self.cursor.x = self.rect.left()
        }

        if self.cursor.y + (size.y * self.linear.wrap as i32) > self.rect.bottom() + 1 {
            return None;
        }
        let rect = Rect::from_min_size(self.cursor, size);
        self.cursor.x += size.x + self.linear.spacing.x;
        self.max = self.max.max(size);
        Some(rect)
    }

    fn vertical(&mut self, size: Vec2) -> Option<Rect> {
        // let (x, y) = (self.linear.x, self.linear.y);

        if self.cursor.y + size.y > self.rect.bottom() {
            if !self.linear.wrap {
                return None;
            }
            self.cursor.x += self.linear.spacing.x + size.x.max(self.max.x);
            self.cursor.y = self.rect.top()
        }
        if self.cursor.x + (size.x * self.linear.wrap as i32) > self.rect.right() + 1 {
            return None;
        }
        let rect = Rect::from_min_size(self.cursor, size);
        self.cursor.y += size.y + self.linear.spacing.y;
        self.max = self.max.max(size);
        Some(rect)
    }
}

// TODO clipping
// TODO what should clipping actually do?

pub struct LinearLayout {
    direction: Direction,
    wrap: bool,
    spacing: Vec2,
}

impl Default for LinearLayout {
    fn default() -> Self {
        Self::DEFAULT
    }
}

impl LinearLayout {
    const DEFAULT: Self = Self {
        direction: Direction::Horizontal,
        wrap: false,
        spacing: Vec2::ZERO,
    };

    pub const fn new(direction: Direction) -> Self {
        Self {
            direction,
            ..Self::DEFAULT
        }
    }

    pub const fn direction(mut self, direction: Direction) -> Self {
        self.direction = direction;
        self
    }

    pub const fn horizontal() -> Self {
        Self::DEFAULT
    }

    pub const fn vertical() -> Self {
        Self {
            direction: Direction::Vertical,
            ..Self::DEFAULT
        }
    }

    pub const fn wrap(mut self, wrap: bool) -> Self {
        self.wrap = wrap;
        self
    }

    pub const fn spacing(mut self, spacing: Vec2) -> Self {
        self.spacing = spacing;
        self
    }

    pub const fn layout(self, rect: Rect) -> LinearAllocator {
        LinearAllocator {
            linear: self,
            cursor: rect.left_top(),
            rect,
            max: Vec2::ZERO,
        }
    }
}
