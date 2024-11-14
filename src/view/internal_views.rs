use crate::{
    layout::Axis,
    math::{Size, Space},
    view::{Builder, Layout, View},
};

#[derive(Debug)]
pub struct Root;
impl View for Root {
    type Args<'v> = ();
    type Response = ();

    fn create(_: Self::Args<'_>) -> Self {
        Self
    }

    fn primary_axis(&self) -> Axis {
        Axis::Vertical
    }

    fn layout(&mut self, mut layout: Layout, space: Space) -> Size {
        layout.new_layer();
        self.default_layout(layout, space.loosen());
        space.max
    }
}

#[derive(Debug)]
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

#[derive(Debug)]
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

#[derive(Debug)]
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
