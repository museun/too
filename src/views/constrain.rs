use crate::{
    math::{Size, Space},
    view::{Builder, Layout, View},
};

#[derive(Debug, Copy, Clone)]
#[must_use = "a view does nothing unless `show()` or `show_children()` is called"]
pub struct Constrain {
    space: Space,
}

impl Constrain {
    pub fn new(space: impl Into<Space>) -> Self {
        Constrain {
            space: space.into(),
        }
    }

    pub fn with(self, other: Self) -> Self {
        Self::new(self.space.constrain(other.space))
    }

    pub fn exact_size(size: impl Into<Size>) -> Self {
        Self::new(Space::from_size(size.into()))
    }

    pub fn max_size(size: impl Into<Size>) -> Self {
        Self::new(Space::new(Size::ZERO, size.into()))
    }

    pub fn min_size(size: impl Into<Size>) -> Self {
        Self::new(Space::new(size.into(), Size::FILL))
    }

    // TODO support floats
    pub fn exact_height(height: i32) -> Self {
        let mut space = Space::UNBOUNDED;
        space.min.height = height as f32;
        space.max.height = height as f32;
        Self::new(space)
    }

    pub fn exact_width(width: i32) -> Self {
        let mut space = Space::UNBOUNDED;
        space.min.width = width as f32;
        space.max.width = width as f32;
        Self::new(space)
    }

    pub fn min_width(min_width: i32) -> Self {
        let mut space = Space::UNBOUNDED;
        space.min.width = min_width as f32;
        Self::new(space)
    }

    pub fn max_width(max_width: i32) -> Self {
        let mut space = Space::UNBOUNDED;
        space.max.width = max_width as f32;
        Self::new(space)
    }

    pub fn min_height(min_height: i32) -> Self {
        let mut space = Space::UNBOUNDED;
        space.min.height = min_height as f32;
        Self::new(space)
    }

    pub fn max_height(max_height: i32) -> Self {
        let mut space = Space::UNBOUNDED;
        space.max.height = max_height as f32;
        Self::new(space)
    }
}

impl<'v> Builder<'v> for Constrain {
    type View = Self;
}

impl View for Constrain {
    type Args<'v> = Self;
    type Response = ();

    fn create(this: Self::Args<'_>) -> Self {
        this
    }

    fn layout(&mut self, layout: Layout, space: Space) -> Size {
        let constrained = self.space.constrain(space);
        self.default_layout(layout, constrained)
    }
}

#[derive(Copy, Clone, Default, Debug, PartialEq)]
pub struct Unconstrained {
    pub horizontal: bool,
    pub vertical: bool,
}

impl Unconstrained {
    pub const fn new() -> Self {
        Self {
            horizontal: false,
            vertical: false,
        }
    }

    pub const fn both() -> Self {
        Self {
            horizontal: true,
            vertical: true,
        }
    }

    pub const fn horizontal(mut self, horizontal: bool) -> Self {
        self.horizontal = horizontal;
        self
    }

    pub const fn vertical(mut self, vertical: bool) -> Self {
        self.vertical = vertical;
        self
    }
}

impl<'v> Builder<'v> for Unconstrained {
    type View = Self;
}

impl View for Unconstrained {
    type Args<'v> = Self;
    type Response = ();

    fn create(args: Self::Args<'_>) -> Self {
        args
    }

    fn layout(&mut self, mut layout: Layout, space: Space) -> Size {
        let node = layout.nodes.get_current();

        let (min_x, max_x) = if self.horizontal {
            (0.0, space.max.width)
        } else {
            (0.0, f32::INFINITY)
        };

        let (min_y, max_y) = if self.vertical {
            (0.0, space.max.height)
        } else {
            (0.0, f32::INFINITY)
        };

        let new = Space::new(Size::new(min_x, min_y), Size::new(max_x, max_y));

        let mut size = Size::ZERO;
        for &child in &node.children {
            size = size.max(layout.compute(child, new))
        }

        new.constrain_min(size)
    }
}
