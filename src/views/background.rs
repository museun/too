use crate::{
    view::{Builder, Render, View},
    Rgba,
};

#[derive(Copy, Clone)]
#[must_use = "a view does nothing unless `show()` or `show_children()` is called"]
pub struct Background {
    bg: Rgba,
}

impl std::fmt::Debug for Background {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.bg.fmt(f)
    }
}

impl Background {
    pub fn new(bg: impl Into<Rgba>) -> Self {
        Self { bg: bg.into() }
    }
}

impl<'v> Builder<'v> for Background {
    type View = Self;
}

impl View for Background {
    type Args<'v> = Self;
    type Response = ();

    fn create(args: Self::Args<'_>) -> Self {
        args
    }

    fn draw(&mut self, mut render: Render) {
        render.fill_bg(self.bg);
        self.default_draw(render);
    }
}
