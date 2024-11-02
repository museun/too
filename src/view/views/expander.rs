use crate::{
    view::{
        geom::{Flex, Size, Space},
        Builder, Elements, Layout, Render, View,
    },
    Pixel,
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
        let axis = layout.parent_axis();
        axis.pack(axis.main(space.max), 0.0)
    }
}

#[derive(Debug, Copy, Clone)]
#[must_use = "a view does nothing unless `show()` or `show_children()` is called"]
pub struct Separator;

impl<'v> Builder<'v> for Separator {
    type View = Self;
}

impl View for Separator {
    type Args<'v> = Self;
    type Response = ();

    fn create(args: Self::Args<'_>) -> Self {
        args
    }

    fn flex(&self) -> Flex {
        Flex::Loose(1.0)
    }

    fn layout(&mut self, layout: Layout, space: Space) -> Size {
        let axis = layout.parent_axis();
        axis.pack(1.0, axis.cross(space.max))
    }

    fn draw(&mut self, mut render: Render) {
        let axis = render.parent_axis();

        let dash = axis.cross((
            Elements::THICK_DASH_HORIZONTAL_LINE,
            Elements::THICK_DASH_VERTICAL_LINE,
        ));

        render.surface.fill_with(Pixel::new(dash).fg("#FFF"));
    }
}
