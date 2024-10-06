use std::ops::RangeInclusive;

use too::Pixel;

use crate::{
    geom::{Point, Size, Space},
    view::Context,
    DrawCtx, Elements, Event, EventCtx, FilledProperty, Handled, HeightProperty, Interest, Knob,
    LayoutCtx, Response, UnfilledProperty, UpdateCtx, View, ViewExt, WidthProperty,
};

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct SliderResponse {
    pub changed: bool,
}

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
    previous: f32,
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
    type Response = SliderResponse;

    fn create(args: Self::Args<'_>) -> Self {
        Self {
            params: args,
            previous: 0.0,
        }
    }

    fn update(&mut self, ctx: UpdateCtx<T>, args: Self::Args<'_>) -> Self::Response {
        self.params = args;
        let value = *(self.params)(ctx.state).value;
        SliderResponse {
            changed: value != self.previous,
        }
    }

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
        self.previous = *params.value;

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

    fn draw(&mut self, mut ctx: DrawCtx<T>) {
        let params = (self.params)(ctx.state);

        // TODO axis
        let (min, max) = (ctx.rect.left(), ctx.rect.right() - 1.0);
        // we need to round to the next step
        let x = Self::normalize(*params.value, &params.range);
        let x = min + (x * (max - min));

        let pixel = Pixel::new(ctx.properties.unfilled::<Slider>()).fg(ctx.theme.outline);
        ctx.surface.fill(pixel);

        let track = ctx.properties.filled::<Slider>();
        let pixel = Pixel::new(track).fg(ctx.theme.contrast);

        let pos = x - ctx.rect.left();
        ctx.surface.horizontal_fill(0.0, pos, pixel);

        let point = Point::new(pos, 0.0);
        let &knob = ctx.properties.get_or_default::<Knob>();
        let pixel = Pixel::new(knob).fg(ctx.theme.primary);
        ctx.surface.set(point, pixel);
    }
}

// TODO this should return a bool if changed
pub fn slider<T: 'static>(
    ctx: &mut Context<T>,
    params: fn(&mut T) -> SliderParams<'_>,
) -> Response<SliderResponse> {
    Slider::show(params, ctx)
}
