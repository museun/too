use crate::{
    geom::{Size, Space},
    response::UserResponse,
    view::Context,
    LayoutCtx, NoResponse, UpdateCtx, View, ViewExt,
};

struct Constrain {
    space: Space,
}

impl<T: 'static> View<T> for Constrain {
    type Args<'a> = Space;
    type Response = NoResponse;

    fn create(args: Self::Args<'_>) -> Self {
        Self { space: args }
    }

    fn update(&mut self, ctx: UpdateCtx<T>, args: Self::Args<'_>) -> Self::Response {
        self.space = args
    }

    fn layout(&mut self, ctx: LayoutCtx<T>, space: Space) -> Size {
        let space = self.space.constrain(space);
        self.default_layout(ctx, space)
    }
}

pub fn constrain<T: 'static, R>(
    space: impl Into<Space>,
    ctx: &mut Context<T>,
    show: impl FnOnce(&mut Context<T>) -> R,
) -> UserResponse<R> {
    let space = space.into();
    Constrain::show_children(space, ctx, show)
}

pub fn size<T: 'static, R>(
    size: impl Into<Size>,
    ctx: &mut Context<T>,
    show: impl FnOnce(&mut Context<T>) -> R,
) -> UserResponse<R> {
    let size = size.into();
    Constrain::show_children(Space::from_size(size), ctx, show)
}

pub fn min_size<T: 'static, R>(
    min_size: impl Into<Size>,
    ctx: &mut Context<T>,
    show: impl FnOnce(&mut Context<T>) -> R,
) -> UserResponse<R> {
    let min_size = min_size.into();
    let space = Space::new(min_size, Size::FILL);
    Constrain::show_children(space, ctx, show)
}

pub fn max_size<T: 'static, R>(
    max_size: impl Into<Size>,
    ctx: &mut Context<T>,
    show: impl FnOnce(&mut Context<T>) -> R,
) -> UserResponse<R> {
    let max_size = max_size.into();
    let space = Space::new(Size::ZERO, max_size);
    Constrain::show_children(space, ctx, show)
}

pub fn width<T: 'static, R>(
    width: f32,
    ctx: &mut Context<T>,
    show: impl FnOnce(&mut Context<T>) -> R,
) -> UserResponse<R> {
    let mut space = Space::UNBOUNDED;
    space.min.width = width;
    space.max.width = width;
    Constrain::show_children(space, ctx, show)
}

pub fn height<T: 'static, R>(
    height: f32,
    ctx: &mut Context<T>,
    show: impl FnOnce(&mut Context<T>) -> R,
) -> UserResponse<R> {
    let mut space = Space::UNBOUNDED;
    space.min.height = height;
    space.max.height = height;
    Constrain::show_children(space, ctx, show)
}

pub fn min_width<T: 'static, R>(
    min_width: f32,
    ctx: &mut Context<T>,
    show: impl FnOnce(&mut Context<T>) -> R,
) -> UserResponse<R> {
    let mut space = Space::UNBOUNDED;
    space.min.width = min_width;
    Constrain::show_children(space, ctx, show)
}

pub fn min_height<T: 'static, R>(
    min_height: f32,
    ctx: &mut Context<T>,
    show: impl FnOnce(&mut Context<T>) -> R,
) -> UserResponse<R> {
    let mut space = Space::UNBOUNDED;
    space.min.height = min_height;
    Constrain::show_children(space, ctx, show)
}

pub fn max_width<T: 'static, R>(
    max_width: f32,
    ctx: &mut Context<T>,
    show: impl FnOnce(&mut Context<T>) -> R,
) -> UserResponse<R> {
    let mut space = Space::UNBOUNDED;
    space.max.width = max_width;
    Constrain::show_children(space, ctx, show)
}

pub fn max_height<T: 'static, R>(
    max_height: f32,
    ctx: &mut Context<T>,
    show: impl FnOnce(&mut Context<T>) -> R,
) -> UserResponse<R> {
    let mut space = Space::UNBOUNDED;
    space.max.height = max_height;
    Constrain::show_children(space, ctx, show)
}
