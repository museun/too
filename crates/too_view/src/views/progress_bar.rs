use std::ops::RangeInclusive;

use too::Pixel;

use crate::{
    geom::{Size, Space},
    view::Context,
    DrawCtx, Elements, FilledProperty, HeightProperty, LayoutCtx, NoResponse, Response,
    UnfilledProperty, UpdateCtx, View, ViewExt, WidthProperty,
};

use super::FillCharacter;

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

    fn draw(&mut self, ctx: DrawCtx<T>) {
        let params = (self.params)(ctx.state);

        let (min, max) = (ctx.rect.left(), ctx.rect.right());
        let x = Self::normalize(*params.value, &params.range);
        let x = min + (x * (max - min));

        ctx.surface.draw(FillCharacter {
            char: ctx.properties.unfilled::<Self>(),
            fg: ctx.theme.outline,
        });

        // surface::crop does not work -- we need to normalize our rect to 0,0
        // TODO axis
        let filled = ctx.properties.filled::<Self>();
        let pixel = Pixel::new(filled).bg(ctx.theme.primary);
        for x in 0..(x - ctx.rect.left()).round() as i32 {
            ctx.surface.put(too::pos2(x, 0), pixel);
        }
    }
}

pub fn progress_bar<T: 'static>(
    ctx: &mut Context<'_, T>,
    params: fn(&mut T) -> ProgressBarParams<'_>,
) -> Response<()> {
    Progress::show(params, ctx)
}
