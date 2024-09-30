use std::ops::RangeInclusive;

use too::Pixel;

use crate::{
    geom::{Point, Size, Space},
    view::Context,
    DrawCtx, Elements, Event, EventCtx, FilledProperty, Handled, HeightProperty, Interest, Knob,
    LayoutCtx, NoResponse, Response, UnfilledProperty, UpdateCtx, View, ViewExt, WidthProperty,
};

// TODO &mut &f32 could do disable
pub struct SliderParams<'a> {
    pub value: &'a mut f32,
    pub range: RangeInclusive<f32>,
}

impl<'a> SliderParams<'a> {
    pub fn new(value: &'a mut f32) -> Self {
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

pub struct Slider<T: 'static = ()> {
    params: fn(&mut T) -> SliderParams<'_>,
}

impl WidthProperty for Slider<()> {
    const WIDTH: f32 = 20.0;
}

impl HeightProperty for Slider<()> {
    const HEIGHT: f32 = 1.0;
}

impl FilledProperty for Slider<()> {
    const FILLED: char = Elements::THICK_HORIZONTAL_LINE;
}

impl UnfilledProperty for Slider<()> {
    const UNFILLED: char = Elements::THICK_DASH_HORIZONTAL_LINE;
}

impl<T: 'static> Slider<T> {
    fn normalize(value: f32, range: &RangeInclusive<f32>) -> f32 {
        let value = value.clamp(*range.start(), *range.end());
        (value - range.start()) / (range.end() - range.start())
    }

    fn denormalize(value: f32, range: &RangeInclusive<f32>) -> f32 {
        let value = value.clamp(0.0, 1.0);
        value * (range.end() - range.start()) + range.start()
    }
}

impl<T: 'static> View<T> for Slider<T> {
    type Args<'a> = fn(&mut T) -> SliderParams<'_>;
    type Response = NoResponse;

    fn create(args: Self::Args<'_>) -> Self {
        Self { params: args }
    }

    fn update(&mut self, ctx: UpdateCtx<T>, args: Self::Args<'_>) -> Self::Response {}

    fn interest(&self) -> Interest {
        Interest::MOUSE
    }

    fn event(&mut self, ctx: EventCtx<T>, event: &Event) -> Handled {
        let Event::MouseDrag(ev) = event else {
            return Handled::Bubble;
        };

        let (min, max) = (ctx.rect.left(), ctx.rect.right());
        // TODO axis

        // we need to round to the next step
        let params = (self.params)(ctx.state);
        let p = (ev.pos.x - min) / (max - min);
        *params.value = Self::denormalize(p, &params.range);

        Handled::Sink
    }

    fn layout(&mut self, ctx: LayoutCtx<T>, space: Space) -> Size {
        // TODO axis
        Size::new(
            ctx.properties.width::<Slider>(),
            ctx.properties.height::<Slider>(),
        )
    }

    fn draw(&mut self, ctx: DrawCtx<T>) {
        let params = (self.params)(ctx.state);

        // TODO axis
        let (min, max) = (ctx.rect.left(), ctx.rect.right() - 1.0);
        // we need to round to the next step
        let x = Self::normalize(*params.value, &params.range);
        let x = min + (x * (max - min));

        let pixel = Pixel::new(ctx.properties.unfilled::<Slider>()).fg(ctx.theme.outline);
        ctx.surface.draw(pixel);

        let track = ctx.properties.filled::<Slider>();
        let pixel = Pixel::new(track).fg(ctx.theme.contrast);

        // TODO make helpers for this
        // surface::crop does not work -- we need to normalize our rect to 0,0
        for x in 0..(x - ctx.rect.left()).round() as i32 {
            ctx.surface.put(too::math::pos2(x, 0), pixel);
        }

        let point = Point::new(x - ctx.rect.left(), 0.0);
        let &knob = ctx.properties.get_or_default::<Knob>();
        ctx.surface
            .put(point.into(), Pixel::new(knob).fg(ctx.theme.primary));
    }
}

pub fn slider<T: 'static>(
    ctx: &mut Context<T>,
    params: fn(&mut T) -> SliderParams<'_>,
) -> Response<()> {
    Slider::show(params, ctx)
}
