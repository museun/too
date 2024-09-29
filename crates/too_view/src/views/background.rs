use too::{shapes::Fill, Rgba};

use crate::{response::UserResponse, view::Context, DrawCtx, NoResponse, UpdateCtx, View, ViewExt};

struct Background {
    bg: Rgba,
}

impl<T: 'static> View<T> for Background {
    type Args<'a> = Rgba;
    type Response = NoResponse;

    fn create(args: Self::Args<'_>) -> Self {
        Self { bg: args }
    }

    fn update(&mut self, ctx: UpdateCtx<T>, args: Self::Args<'_>) -> Self::Response {
        self.bg = args;
    }

    fn draw(&mut self, ctx: DrawCtx<T>) {
        ctx.surface.draw(Fill::new(self.bg));
        self.default_draw(ctx);
    }
}

pub fn background<T: 'static, R>(
    bg: impl Into<Rgba>,
    ctx: &mut Context<'_, T>,
    show: impl FnOnce(&mut Context<'_, T>) -> R,
) -> UserResponse<R> {
    Background::show_children(bg.into(), ctx, show)
}
