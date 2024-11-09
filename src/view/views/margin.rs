use crate::view::{
    geom::{self, Size, Space},
    Builder, Layout, View,
};

#[derive(Copy, Clone, Debug, PartialEq)]
#[must_use = "a view does nothing unless `show()` or `show_children()` is called"]
pub struct Margin {
    margin: geom::Margin,
}

impl Margin {
    pub fn new(margin: impl Into<geom::Margin>) -> Self {
        Self {
            margin: margin.into(),
        }
    }
}

impl<'v> Builder<'v> for Margin {
    type View = Self;
}

impl View for Margin {
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
            layout.layout.set_position(child, offset);
        }
        size
    }
}
