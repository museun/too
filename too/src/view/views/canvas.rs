use crate::CroppedSurface;

use super::super::{
    geom::{Size, Space},
    view::Context,
    DrawCtx, LayoutCtx, UpdateCtx, View, ViewExt,
};

struct Canvas<T: 'static> {
    draw: fn(&mut T, &mut CroppedSurface<'_>),
}

impl<T: 'static> View<T> for Canvas<T> {
    type Args<'a> = fn(&mut T, &mut CroppedSurface<'_>);
    type Response = ();

    fn create(args: Self::Args<'_>) -> Self {
        Self { draw: args }
    }

    fn update(&mut self, ctx: UpdateCtx<T>, args: Self::Args<'_>) -> Self::Response {
        self.draw = args
    }

    fn layout(&mut self, ctx: LayoutCtx<T>, space: Space) -> Size {
        space.max
    }

    fn draw(&mut self, mut ctx: DrawCtx<T>) {
        // TODO this needs to be the parent rect
        let mut surface = CroppedSurface::new(ctx.rect.into(), ctx.surface.surface_raw());
        (self.draw)(ctx.state, &mut surface);
    }
}

pub fn canvas<T: 'static>(ctx: &mut Context<T>, draw: fn(&mut T, &mut CroppedSurface<'_>)) {
    Canvas::show(draw, ctx);
}
