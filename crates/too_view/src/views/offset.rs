use crate::{
    geom::{Point, Size, Space},
    view::Context,
    LayoutCtx, UpdateCtx, View, ViewExt,
};

struct Offset<T: 'static> {
    args: fn(&T) -> Point,
}

impl<T: 'static> View<T> for Offset<T> {
    type Args<'a> = fn(&T) -> Point;
    type Response = ();

    fn create(args: Self::Args<'_>) -> Self {
        Self { args }
    }

    fn update(&mut self, ctx: UpdateCtx<T>, args: Self::Args<'_>) -> Self::Response {
        self.args = args;
    }

    fn layout(&mut self, mut ctx: LayoutCtx<T>, space: Space) -> Size {
        let mut size = Size::ZERO;
        let offset = (self.args)(ctx.state);
        for &child in ctx.children {
            size = size.max(ctx.compute_layout(child, space));
            ctx.translate_pos(child, offset);
        }
        size
    }
}

pub fn offset<T: 'static, R>(
    pos: fn(&T) -> Point,
    ctx: &mut Context<T>,
    show: fn(&mut Context<T>) -> R,
) -> R {
    let (_, resp) = Offset::show_children(pos, ctx, show);
    resp
}
