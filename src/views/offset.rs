use crate::{
    math::{Pos2, Size, Space},
    view::{Builder, Layout, View},
};

#[derive(Copy, Clone, Debug, PartialEq)]
#[must_use = "a view does nothing unless `show()` or `show_children()` is called"]
pub struct Offset {
    pos: Pos2,
}

impl Offset {
    pub fn new(pos: impl Into<Pos2>) -> Self {
        Self { pos: pos.into() }
    }
}

impl<'v> Builder<'v> for Offset {
    type View = Self;
}

impl View for Offset {
    type Args<'v> = Self;
    type Response = ();

    fn create(args: Self::Args<'_>) -> Self {
        args
    }

    fn layout(&mut self, mut layout: Layout, space: Space) -> Size {
        let node = layout.nodes.get_current();
        let mut size = Size::ZERO;
        for &child in &node.children {
            size = size.max(layout.compute(child, space));
            layout.set_position(child, self.pos);
        }
        space.constrain_min(size)
    }
}
