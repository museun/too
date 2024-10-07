use too::{Pixel, Rgba};

use crate::{view::Context, DrawCtx, UpdateCtx, View, ViewExt};

struct Background {
    bg: Rgba,
}

impl<T: 'static> View<T> for Background {
    type Args<'a> = Rgba;
    type Response = ();

    fn create(args: Self::Args<'_>) -> Self {
        Self { bg: args }
    }

    fn update(&mut self, ctx: UpdateCtx<T>, args: Self::Args<'_>) -> Self::Response {
        self.bg = args
    }

    fn draw(&mut self, mut ctx: DrawCtx<T>) {
        ctx.surface.fill(self.bg);
        self.default_draw(ctx);
    }
}

pub fn background<T: 'static, R>(
    ctx: &mut Context<T>,
    bg: impl Into<Rgba>,
    show: impl FnOnce(&mut Context<T>) -> R,
) -> R {
    let (_, resp) = Background::show_children(bg.into(), ctx, show);
    resp
}

struct Fill {
    pixel: Pixel,
}

impl<T: 'static> View<T> for Fill {
    type Args<'a> = Pixel;
    type Response = ();

    fn create(args: Self::Args<'_>) -> Self {
        Self { pixel: args }
    }

    fn update(&mut self, ctx: UpdateCtx<T>, args: Self::Args<'_>) -> Self::Response {
        self.pixel = args;
    }

    fn layout(&mut self, ctx: crate::LayoutCtx<T>, space: crate::geom::Space) -> crate::geom::Size {
        space.max
    }

    fn draw(&mut self, mut ctx: DrawCtx<T>) {
        ctx.surface.fill(self.pixel);
    }
}

pub fn fill<T: 'static>(ctx: &mut Context<T>, pixel: impl Into<Pixel>) {
    Fill::show(pixel.into(), ctx)
}
