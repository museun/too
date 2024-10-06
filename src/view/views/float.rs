// keep me

use crate::view::{
    geom::{Size, Space},
    Builder, Layout, View,
};

#[derive(Copy, Clone, Debug, Default, PartialEq)]
#[must_use = "a view does nothing unless `show()` or `show_children()` is called"]
pub struct Float;

impl<'v> Builder<'v> for Float {
    type View = Self;
}

impl View for Float {
    type Args<'v> = Self;
    type Response = ();

    fn create(args: Self::Args<'_>) -> Self {
        args
    }

    fn layout(&mut self, mut layout: Layout, space: Space) -> Size {
        layout.new_layer();
        self.default_layout(layout, Space::tight(space.size()))
    }
}

pub const fn float() -> Float {
    Float
}

#[derive(Copy, Clone, Debug, Default, PartialEq)]
#[must_use = "a view does nothing unless `show()` or `show_children()` is called"]
pub struct Clip;

impl<'v> Builder<'v> for Clip {
    type View = Self;
}

impl View for Clip {
    type Args<'v> = Self;
    type Response = ();

    fn create(args: Self::Args<'_>) -> Self {
        args
    }

    fn layout(&mut self, mut layout: Layout, space: Space) -> Size {
        layout.enable_clipping();
        self.default_layout(layout, Space::tight(space.size()))
    }
}

pub const fn clip() -> Clip {
    Clip
}
