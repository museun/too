use crate::{
    geom::{Size, Space},
    view::Context,
    LayoutCtx, UpdateCtx, View, ViewExt,
};

struct Constrain {
    space: Space,
}

impl<T: 'static> View<T> for Constrain {
    type Args<'a> = Space;
    type Response = ();

    fn create(args: Self::Args<'_>) -> Self {
        Self { space: args }
    }

    fn update(&mut self, ctx: UpdateCtx<T>, args: Self::Args<'_>) -> Self::Response {
        self.space = args
    }

    fn layout(&mut self, ctx: LayoutCtx<T>, space: Space) -> Size {
        let constrained = self.space.constrain(space);
        // eprintln!(
        //     "constrain({:?})\n\
        //      available_space: {space:?}\n\
        //      requested space: {:?}\n\
        //      constrained space: {constrained:?}",
        //     ctx.current_id, self.space
        // );

        self.default_layout(ctx, constrained)
    }
}

pub fn constrain<T: 'static, R>(
    space: impl Into<Space>,
    ctx: &mut Context<T>,
    show: impl FnOnce(&mut Context<T>) -> R,
) -> R {
    let (_, resp) = Constrain::show_children(space.into(), ctx, show);
    resp
}

pub fn size<T: 'static, R>(
    size: impl Into<Size>,
    ctx: &mut Context<T>,
    show: impl FnOnce(&mut Context<T>) -> R,
) -> R {
    let space = Space::from_size(size.into());
    let (_, resp) = Constrain::show_children(space, ctx, show);
    resp
}

pub fn min_size<T: 'static, R>(
    min_size: impl Into<Size>,
    ctx: &mut Context<T>,
    show: impl FnOnce(&mut Context<T>) -> R,
) -> R {
    let space = Space::new(min_size.into(), Size::FILL);
    let (_, resp) = Constrain::show_children(space, ctx, show);
    resp
}

pub fn max_size<T: 'static, R>(
    max_size: impl Into<Size>,
    ctx: &mut Context<T>,
    show: impl FnOnce(&mut Context<T>) -> R,
) -> R {
    let space = Space::new(Size::ZERO, max_size.into());
    let (_, resp) = Constrain::show_children(space, ctx, show);
    resp
}

pub fn width<T: 'static, R>(
    width: f32,
    ctx: &mut Context<T>,
    show: impl FnOnce(&mut Context<T>) -> R,
) -> R {
    let mut space = Space::UNBOUNDED;
    space.min.width = width;
    space.max.width = width;
    let (_, resp) = Constrain::show_children(space, ctx, show);
    resp
}

pub fn height<T: 'static, R>(
    height: f32,
    ctx: &mut Context<T>,
    show: impl FnOnce(&mut Context<T>) -> R,
) -> R {
    let mut space = Space::UNBOUNDED;
    space.min.height = height;
    space.max.height = height;
    let (_, resp) = Constrain::show_children(space, ctx, show);
    resp
}

pub fn min_width<T: 'static, R>(
    min_width: f32,
    ctx: &mut Context<T>,
    show: impl FnOnce(&mut Context<T>) -> R,
) -> R {
    let mut space = Space::UNBOUNDED;
    space.min.width = min_width;
    let (_, resp) = Constrain::show_children(space, ctx, show);
    resp
}

pub fn min_height<T: 'static, R>(
    min_height: f32,
    ctx: &mut Context<T>,
    show: impl FnOnce(&mut Context<T>) -> R,
) -> R {
    let mut space = Space::UNBOUNDED;
    space.min.height = min_height;
    let (_, resp) = Constrain::show_children(space, ctx, show);
    resp
}

pub fn max_width<T: 'static, R>(
    max_width: f32,
    ctx: &mut Context<T>,
    show: impl FnOnce(&mut Context<T>) -> R,
) -> R {
    let mut space = Space::UNBOUNDED;
    space.max.width = max_width;
    let (_, resp) = Constrain::show_children(space, ctx, show);
    resp
}

pub fn max_height<T: 'static, R>(
    max_height: f32,
    ctx: &mut Context<T>,
    show: impl FnOnce(&mut Context<T>) -> R,
) -> R {
    let mut space = Space::UNBOUNDED;
    space.max.height = max_height;
    let (_, resp) = Constrain::show_children(space, ctx, show);
    resp
}
