use crate::{
    view::{
        geom::{Size, Space},
        Builder, Layout, Render, View,
    },
    Rgba,
};

#[derive(Copy, Clone, Debug, PartialEq)]
#[must_use = "a view does nothing unless `show()` or `show_children()` is called"]
pub struct Fill {
    bg: Option<Rgba>,
    size: Size,
}

impl Fill {
    pub fn new(bg: impl Into<Rgba>, size: impl Into<Size>) -> Self {
        Self {
            bg: Some(bg.into()),
            size: size.into(),
        }
    }

    pub fn fill(bg: impl Into<Rgba>) -> Self {
        Self::new(bg, Size::FILL)
    }

    pub const fn all_space() -> Self {
        Self {
            bg: None,
            size: Size::FILL,
        }
    }
}

impl<'v> Builder<'v> for Fill {
    type View = Self;
}

impl View for Fill {
    type Args<'v> = Self;
    type Response = ();

    fn create(args: Self::Args<'_>) -> Self {
        args
    }

    fn layout(&mut self, layout: Layout, space: Space) -> Size {
        space.fit(self.size)
    }

    fn draw(&mut self, mut render: Render) {
        if let Some(bg) = self.bg {
            render.surface.fill(bg);
        }
    }
}

pub fn fill(bg: impl Into<Rgba>, size: impl Into<Size>) -> Fill {
    Fill::new(bg, size)
}
