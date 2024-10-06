use crate::view::{
    geom::{Flex, Size, Space},
    Builder, Layout, View,
};

#[derive(Debug, Copy, Clone)]
#[must_use = "a view does nothing unless `show()` or `show_children()` is called"]
pub struct Expander;

impl<'v> Builder<'v> for Expander {
    type View = Self;
}

impl View for Expander {
    type Args<'v> = Self;
    type Response = ();

    fn create(args: Self::Args<'_>) -> Self {
        args
    }

    fn flex(&self) -> Flex {
        Flex::Tight(1.0)
    }

    fn layout(&mut self, layout: Layout, space: Space) -> Size {
        // TODO get axis from parent
        space.max
        // let Some(axis) = layout.parent_axis() else {
        //     return space.max;
        // };

        // match axis {
        //     Axis::Horizontal => Size::new(space.max.width, 0.0),
        //     Axis::Vertical => Size::new(0.0, space.max.height),
        // }
    }
}
