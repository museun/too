use crate::{
    layout::Align2,
    math::{Size, Space},
    view::{Builder, Layout, View},
};

pub const fn aligned(align: Align2) -> Aligned {
    Aligned { align }
}

#[derive(Debug, Copy, Clone)]
#[must_use = "a view does nothing unless `show()` or `show_children()` is called"]
pub struct Aligned {
    align: Align2,
}

impl<'v> Builder<'v> for Aligned {
    type View = Self;
}

impl View for Aligned {
    type Args<'v> = Self;
    type Response = ();

    fn create(this: Self::Args<'_>) -> Self {
        this
    }

    fn layout(&mut self, mut layout: Layout, space: Space) -> Size {
        let mut size = space.size();

        let child_space = space.loosen();
        let node = layout.nodes.get_current();
        for &child in &node.children {
            let next = layout.compute(child, child_space);
            size = size.max(next);

            let pos = size * self.align - next * self.align;
            layout.set_position(child, pos);
        }

        size.max(space.min.finite_or_zero())
            .max(space.max.finite_or_zero())
    }
}
