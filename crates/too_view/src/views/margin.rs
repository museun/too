use crate::{
    geom::{self, Size, Space},
    response::UserResponse,
    view::Context,
    LayoutCtx, NoResponse, UpdateCtx, View, ViewExt,
};

struct Margin {
    margin: geom::Margin,
}

impl<T: 'static> View<T> for Margin {
    type Args<'a> = geom::Margin;
    type Response = NoResponse;

    fn create(args: Self::Args<'_>) -> Self {
        Self { margin: args }
    }

    fn update(&mut self, ctx: UpdateCtx<T>, args: Self::Args<'_>) -> Self::Response {
        self.margin = args;
    }

    fn layout(&mut self, mut ctx: LayoutCtx<T>, space: Space) -> Size {
        let margin = self.margin.sum();
        let offset = self.margin.left_top().to_point();
        let space = Space {
            min: (space.min - margin).max(Size::ZERO),
            max: (space.max - margin).max(Size::ZERO),
        };

        let mut size = Size::ZERO;
        for &child in ctx.children {
            size = ctx.compute_layout(child, space) + margin;
            ctx.translate_pos(child, offset);
        }
        space.min.max(size.max(margin))
    }
}

pub fn margin<T: 'static, R>(
    margin: impl Into<geom::Margin>,
    ctx: &mut Context<'_, T>,
    show: impl FnOnce(&mut Context<'_, T>) -> R,
) -> UserResponse<R> {
    Margin::show_children(margin.into(), ctx, show)
}
