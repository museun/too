use crate::Pixel;

use super::super::{
    elements::Elements,
    geom::{Size, Space},
    properties::*,
    view::Context,
    DrawCtx, LayoutCtx, UpdateCtx, View, ViewExt,
};

// TODO range and step by
pub struct ProgressBarParams<'a> {
    pub value: Option<&'a f32>,
}

impl<'a> ProgressBarParams<'a> {
    pub fn new(value: impl Into<Option<&'a f32>>) -> Self {
        Self {
            value: value.into(),
        }
    }
}

impl WidthProperty for Progress {
    const WIDTH: f32 = 20.0;
}

impl HeightProperty for Progress {
    const HEIGHT: f32 = 1.0;
}

impl FilledProperty for Progress {
    const FILLED: char = Elements::LARGE_RECT;
}

impl UnfilledProperty for Progress {
    const UNFILLED: char = Elements::LARGE_RECT;
}

struct Progress<T: 'static = ()> {
    params: fn(&mut T) -> ProgressBarParams<'_>,
}

impl<T: 'static> View<T> for Progress<T> {
    type Args<'a> = fn(&mut T) -> ProgressBarParams<'_>;
    type Response = ();

    fn create(args: Self::Args<'_>) -> Self {
        Self { params: args }
    }

    fn update(&mut self, ctx: UpdateCtx<T>, args: Self::Args<'_>) -> Self::Response {
        self.params = args
    }

    fn layout(&mut self, ctx: LayoutCtx<T>, space: Space) -> Size {
        // TODO axis
        Size::new(
            ctx.properties.width::<Progress>(),
            ctx.properties.height::<Progress>(),
        )
    }

    fn draw(&mut self, mut ctx: DrawCtx<T>) {
        let params = (self.params)(ctx.state);

        let (min, max) = (ctx.rect.left(), ctx.rect.right() - 1.0);
        let Some(&nx) = params.value else { return };

        let x = min + (nx * (max - min)) - ctx.rect.left();

        let unfilled = ctx.properties.unfilled::<Progress>();
        let pixel = Pixel::new(unfilled).fg(ctx.theme.outline);
        ctx.surface.fill(pixel);

        if nx == 0.0 {
            return;
        }

        let filled = ctx.properties.filled::<Progress>();
        let pixel = Pixel::new(filled).fg(ctx.theme.primary);

        if nx == 1.0 {
            ctx.surface.fill(pixel);
            return;
        }

        // TODO axis

        ctx.surface.horizontal_fill((0.0, x), pixel);
    }
}

pub fn progress_bar<T: 'static>(ctx: &mut Context<T>, params: fn(&mut T) -> ProgressBarParams<'_>) {
    Progress::show(params, ctx)
}
