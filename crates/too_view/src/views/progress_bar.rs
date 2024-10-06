use std::ops::RangeInclusive;

use too::Pixel;

use crate::{
    geom::{Size, Space},
    view::Context,
    DrawCtx, Elements, FilledProperty, HeightProperty, LayoutCtx, NoResponse, Response,
    UnfilledProperty, UpdateCtx, View, ViewExt, WidthProperty,
};

pub struct ProgressBarParams<'a> {
    pub value: &'a f32,
    pub range: RangeInclusive<f32>,
}

impl<'a> ProgressBarParams<'a> {
    pub const fn new(value: &'a f32) -> Self {
        Self {
            value,
            range: 0.0..=1.0,
        }
    }

    pub const fn range(mut self, range: RangeInclusive<f32>) -> Self {
        self.range = range;
        self
    }
}

struct Progress<T: 'static> {
    params: fn(&mut T) -> ProgressBarParams<'_>,
}

impl<T: 'static> WidthProperty for Progress<T> {
    const WIDTH: f32 = 20.0;
}

impl<T: 'static> HeightProperty for Progress<T> {
    const HEIGHT: f32 = 1.0;
}

impl<T: 'static> FilledProperty for Progress<T> {
    const FILLED: char = Elements::LARGE_RECT;
}

impl<T: 'static> UnfilledProperty for Progress<T> {
    const UNFILLED: char = Elements::LARGE_RECT;
}

impl<T: 'static> Progress<T> {
    fn normalize(value: f32, range: &RangeInclusive<f32>) -> f32 {
        let value = value.clamp(*range.start(), *range.end());
        (value - range.start()) / (range.end() - range.start())
    }

    fn denormalize(value: f32, range: &RangeInclusive<f32>) -> f32 {
        let value = value.clamp(0.0, 1.0);
        value * (range.end() - range.start()) + range.start()
    }
}

impl<T: 'static> View<T> for Progress<T> {
    type Args<'a> = fn(&mut T) -> ProgressBarParams<'_>;
    type Response = NoResponse;

    fn create(args: Self::Args<'_>) -> Self {
        Self { params: args }
    }

    fn update(&mut self, ctx: UpdateCtx<T>, args: Self::Args<'_>) -> Self::Response {
        self.params = args
    }

    fn layout(&mut self, ctx: LayoutCtx<T>, space: Space) -> Size {
        // TODO axis
        Size::new(
            ctx.properties.width::<Self>(),
            ctx.properties.height::<Self>(),
        )
    }

    fn draw(&mut self, mut ctx: DrawCtx<T>) {
        let params = (self.params)(ctx.state);

        let (min, max) = (ctx.rect.left(), ctx.rect.right());
        let x = Self::normalize(*params.value, &params.range);
        let x = min + (x * (max - min));

        let unfilled = ctx.properties.unfilled::<Self>();
        let pixel = Pixel::new(unfilled).fg(ctx.theme.outline);
        ctx.surface.fill(pixel);

        // TODO axis
        let filled = ctx.properties.filled::<Self>();
        let pixel = Pixel::new(filled).fg(ctx.theme.primary);
        ctx.surface.horizontal_fill(0.0, x - ctx.rect.left(), pixel);
    }
}

pub fn progress_bar<T: 'static>(
    ctx: &mut Context<'_, T>,
    params: fn(&mut T) -> ProgressBarParams<'_>,
) -> Response<()> {
    Progress::show(params, ctx)
}
