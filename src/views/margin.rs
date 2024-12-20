use crate::{
    math::{Margin, Size, Space},
    view::{Builder, Layout, View},
};

#[derive(Copy, Clone, Debug, PartialEq)]
#[must_use = "a view does nothing unless `show()` or `show_children()` is called"]
pub struct Padding {
    margin: Margin,
}

impl Padding {
    pub fn new(margin: impl Into<Margin>) -> Self {
        Self {
            margin: margin.into(),
        }
    }
}

impl<'v> Builder<'v> for Padding {
    type View = Self;
}

impl View for Padding {
    type Args<'v> = Self;
    type Response = ();

    fn create(this: Self::Args<'_>) -> Self {
        this
    }

    fn layout(&mut self, mut layout: Layout, space: Space) -> Size {
        let node = layout.nodes.get_current();

        // TODO this is off by 1 h
        let margin = self.margin.sum();
        let offset = self.margin.left_top();
        let space = space.shrink(margin);
        let mut size = Size::ZERO;
        for &child in &node.children {
            size = layout.compute(child, space) + margin;
            layout.set_position(child, offset);
        }
        size
    }
}
