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

    fn create((): Self::Args<'_>) -> Self {
        Self
    }

    fn primary_axis(&self) -> Axis {
        Axis::Vertical
    }

    fn layout(&mut self, mut layout: Layout, space: Space) -> Size {
        layout.set_layer(super::Layer::Bottom);
        layout.new_layer();
        self.default_layout(layout, space.loosen());
        space.max
    }
}

#[derive(Debug)]
pub struct Clip;
impl Builder<'_> for Clip {
    type View = Self;
    type Style = ();
}

impl View for Clip {
    type Args<'v> = Self;
    type Response = ();

    fn create(args: Self::Args<'_>) -> Self {
        args
    }

    fn layout(&mut self, mut layout: Layout, space: Space) -> Size {
        layout.enable_clipping();
        space.constrain_min(self.default_layout(layout, space))
    }
}

// TODO this is a bad name, this means input layer not render layer
#[derive(Debug)]
pub struct Layer;
impl Builder<'_> for Layer {
    type View = Self;
    type Style = ();
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
pub struct Float(pub super::Layer);
impl Builder<'_> for Float {
    type View = Self;
    type Style = ();
}

impl View for Float {
    type Args<'v> = Self;
    type Response = ();

    fn create(args: Self::Args<'_>) -> Self {
        args
    }

    fn layout(&mut self, mut layout: Layout, space: Space) -> Size {
        layout.set_layer(self.0);
        layout.new_layer();
        space.constrain_min(self.default_layout(layout, space))
    }
}
