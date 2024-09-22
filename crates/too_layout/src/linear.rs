use too_math::{pos2, Pos2, Rect, Vec2};

use crate::{Anchor, Anchor2, Axis};

/// A linear allocator
///
///
/// This type is crated by a [`LinearLayout`]
///
/// This type allocates sizes in a linear fashion such as:
/// * left to top
/// * top to bottom
/// * right to left
/// * bottom to top

pub struct LinearAllocator {
    state: LinearLayout,
    cursor: Pos2,
    rect: Rect,
    max: Vec2,
    anchor: Anchor2,
}

impl LinearAllocator {
    /// Allocate a new size, returning an rect if possible
    ///
    /// This allows you to store sizes of elements in your state
    /// then using this will give you a 'position' (e.g. a complete rect)
    /// that can be used for futher layouts or drawing
    ///
    /// # Example
    /// ```rust,no_run
    /// let mut sizes = [vec2(5,10), vec2(3,1), vec2(4,4)];
    /// let mut layout = LinearLayout::horizontal().layout(some_rect);
    /// for size in sizes {
    ///     if let Some(rect) = layout.allocate(size) {
    ///         // our rect will have the size, with a computed position
    ///     }
    /// }
    /// ```
    pub fn allocate(&mut self, size: Vec2) -> Option<Rect> {
        match self.state.axis {
            Axis::Horizontal => self.allocate_horizontal(size),
            Axis::Vertical => self.allocate_vertical(size),
        }
    }

    fn allocate_vertical(&mut self, size: Vec2) -> Option<Rect> {
        let (main_sign, cross_sign) = self.anchor.sign(self.state.axis);
        let (main_rect_min, main_rect_max) = match self.anchor.y {
            Anchor::Min => (self.rect.top(), self.rect.bottom()),
            Anchor::Max => (self.rect.bottom(), self.rect.top()),
        };
        let cross_rect_max = match self.anchor.x {
            Anchor::Min => self.rect.right(),
            Anchor::Max => self.rect.left(),
        };

        let next_main_pos = self.cursor.y + main_sign * size.y;
        if self.anchor.y.exceeds_bounds(next_main_pos, main_rect_max) {
            if !self.state.wrap {
                return None;
            }
            self.cursor.y = main_rect_min;
            self.cursor.x += (self.max.x + self.state.spacing.x) * cross_sign;
            self.max = Vec2::ZERO;
        }

        let next_cross_pos = self.cursor.x + (size.x * cross_sign);
        if self.anchor.x.exceeds_bounds(next_cross_pos, cross_rect_max) {
            return None;
        }

        let offset = pos2(
            size.x * self.anchor.x.offset(),
            size.y * self.anchor.y.offset(),
        );
        let rect = Rect::from_min_size(self.cursor - offset, size);

        self.cursor.y += (size.y + self.state.spacing.y) * main_sign;
        self.max.x = self.max.x.max(size.x);

        Some(rect)
    }

    fn allocate_horizontal(&mut self, size: Vec2) -> Option<Rect> {
        let (main_sign, cross_sign) = self.anchor.sign(self.state.axis);
        let (main_rect_min, main_rect_max) = match self.anchor.x {
            Anchor::Min => (self.rect.left(), self.rect.right()),
            Anchor::Max => (self.rect.right(), self.rect.left()),
        };
        let cross_rect_max = match self.anchor.y {
            Anchor::Min => self.rect.bottom(),
            Anchor::Max => self.rect.top(),
        };

        let next_main_pos = self.cursor.x + main_sign * size.x;
        if self.anchor.x.exceeds_bounds(next_main_pos, main_rect_max) {
            if !self.state.wrap {
                return None;
            }
            self.cursor.x = main_rect_min;
            self.cursor.y += (self.max.y + self.state.spacing.y) * cross_sign;
            self.max = Vec2::ZERO;
        }

        let next_cross_pos = self.cursor.y + (size.y * cross_sign);
        if self.anchor.y.exceeds_bounds(next_cross_pos, cross_rect_max) {
            return None;
        }

        let offset = pos2(
            size.x * self.anchor.x.offset(),
            size.y * self.anchor.y.offset(),
        );
        let rect = Rect::from_min_size(self.cursor - offset, size);

        self.cursor.x += (size.x + self.state.spacing.x) * main_sign;
        self.max.y = self.max.y.max(size.y);

        Some(rect)
    }
}

/// A layout that uses a linear, with optional wrapping algorithm
pub struct LinearLayout {
    axis: Axis,
    wrap: bool,
    spacing: Vec2,
    anchor: Anchor2,
}

/// The default configuration for a [`LinearLayout`] is:
/// * horizontal
/// * no wrapping
/// * starts at left-top
impl Default for LinearLayout {
    fn default() -> Self {
        Self::DEFAULT
    }
}

impl LinearLayout {
    const DEFAULT: Self = Self {
        axis: Axis::Horizontal,
        wrap: false,
        spacing: Vec2::ZERO,
        anchor: Anchor2::LEFT_TOP,
    };

    /// Create a new [`LinearLayout`] builder using this [`Axis`]
    pub const fn new(axis: Axis) -> Self {
        Self {
            axis,
            ..Self::DEFAULT
        }
    }

    /// Change the [`Axis`] of this builder
    pub const fn axis(mut self, axis: Axis) -> Self {
        self.axis = axis;
        self
    }

    /// Set the anchor for this builder.
    ///
    /// An anchor is where the layout starts from
    ///
    /// e.g.:
    /// * [`Anchor2::LEFT_TOP`]
    /// * [`Anchor2::RIGHT_BOTTOM`]
    pub const fn anchor(mut self, anchor: Anchor2) -> Self {
        self.anchor = anchor;
        self
    }

    /// Set the horizontal anchor for this builder
    ///
    /// A horizontal anchor where where on the ___x___ axis the layout begins
    pub const fn horizontal_anchor(mut self, anchor: Anchor) -> Self {
        self.anchor.x = anchor;
        self
    }

    /// Set the vertical anchor for this builder
    ///
    /// A vertical anchor where where on the ___y___ axis the layout begins
    pub const fn vertical_anchor(mut self, anchor: Anchor) -> Self {
        self.anchor.y = anchor;
        self
    }

    /// Create a default horizontal layout builder
    pub const fn horizontal() -> Self {
        Self::DEFAULT
    }

    /// Create a default vertical layout builder
    pub const fn vertical() -> Self {
        Self {
            axis: Axis::Vertical,
            ..Self::DEFAULT
        }
    }

    /// Should this layout wrap?
    pub const fn wrap(mut self, wrap: bool) -> Self {
        self.wrap = wrap;
        self
    }

    /// The spacing for the layout.
    ///
    /// Spacing is the gap between 2 elements.
    /// * `spacing.x` is the horizontal spacing
    /// * `spacing.y` is the vertical spacing
    pub const fn spacing(mut self, spacing: Vec2) -> Self {
        self.spacing = spacing;
        self
    }

    /// Construct the [`LinearAllocator`] from this type
    ///
    /// This takes in the target [`Rect`] that the allocator will fit everything into.
    pub fn layout(self, rect: Rect) -> LinearAllocator {
        let cursor = match (self.anchor.x, self.anchor.y) {
            (Anchor::Min, Anchor::Min) => rect.left_top(),
            (Anchor::Min, Anchor::Max) => rect.left_bottom(),
            (Anchor::Max, Anchor::Min) => rect.right_top(),
            (Anchor::Max, Anchor::Max) => rect.right_bottom(),
        };

        LinearAllocator {
            cursor,
            rect,
            max: Vec2::ZERO,
            anchor: self.anchor,
            state: self,
        }
    }
}
