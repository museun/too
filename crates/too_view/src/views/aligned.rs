use too::layout::Align2;

use crate::{
    geom::{Size, Space},
    response::UserResponse,
    view::Context,
    LayoutCtx, NoResponse, UpdateCtx, View, ViewExt,
};

struct Aligned {
    align2: Align2,
}

impl<T: 'static> View<T> for Aligned {
    type Args<'a> = Align2;
    type Response = NoResponse;

    fn create(args: Self::Args<'_>) -> Self {
        Self { align2: args }
    }

    fn update(&mut self, ctx: UpdateCtx<T>, args: Self::Args<'_>) -> Self::Response {
        self.align2 = args;
    }

    fn layout(&mut self, mut ctx: LayoutCtx<T>, space: Space) -> Size {
        let space = space.loosen();

        let mut size = space.size();
        for &child in ctx.children {
            let next = ctx.compute_layout(child, space);
            size = size.max(next);
            let pos = size * self.align2 - next * self.align2;
            ctx.translate_pos(child, pos);
        }

        size.max(space.min.finite_or_zero())
            .max(space.max.finite_or_zero())
    }
}

pub fn center<T: 'static, R>(
    ctx: &mut Context<'_, T>,
    show: impl FnOnce(&mut Context<'_, T>) -> R,
) -> UserResponse<R> {
    align(Align2::CENTER_CENTER, ctx, show)
}

pub fn align<T: 'static, R>(
    align: Align2,
    ctx: &mut Context<'_, T>,
    show: impl FnOnce(&mut Context<'_, T>) -> R,
) -> UserResponse<R> {
    Aligned::show_children(align, ctx, show)
}
