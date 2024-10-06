use crate::view::{
    geom::{Size, Space},
    Builder, Layout, View,
};

#[derive(Debug)]
#[must_use = "a view does nothing unless `show()` or `show_children()` is called"]
pub struct Layer;

impl<'v> Builder<'v> for Layer {
    type View = Self;
}

impl View for Layer {
    type Args<'v> = Self;
    type Response = ();

    fn create(args: Self::Args<'_>) -> Self {
        args
    }

    fn layout(&mut self, mut layout: Layout, space: Space) -> Size {
        layout.new_layer();
        space.constrain_min(self.default_layout(layout, space))
    }
}

#[derive(Debug)]
#[must_use = "a view does nothing unless `show()` or `show_children()` is called"]
pub struct Scope;

impl<'v> Builder<'v> for Scope {
    type View = Self;
}

impl View for Scope {
    type Args<'v> = Self;
    type Response = ();

    fn create(args: Self::Args<'_>) -> Self {
        args
    }
}
