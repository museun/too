use std::ops::RangeInclusive;

use too::Pixel;

use crate::{
    geom::{denormalize, normalize, Point, Size, Space},
    view::Context,
    DrawCtx, Elements, Event, EventCtx, FilledProperty, Handled, HeightProperty, Interest, Knob,
    LayoutCtx, UnfilledProperty, UpdateCtx, View, ViewExt, WidthProperty,
};

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct SliderResponse {
    pub changed: bool,
}

// TODO &mut &f32 could do disable
pub struct SliderParams<'a> {
    pub value: Option<&'a mut f32>,
    pub min: f32,
    pub max: f32,
    pub step: f32,
}

impl<'a> SliderParams<'a> {
    pub fn new(value: impl Into<Option<&'a mut f32>>) -> Self {
        Self {
            value: value.into(),
            min: 0.0,
            max: 1.0,
            step: 0.1,
        }
    }

    pub const fn range(mut self, range: RangeInclusive<f32>) -> Self {
        self.min = *range.start();
        self.max = *range.end();
        self
    }

    pub const fn step_by(mut self, step: f32) -> Self {
        self.step = step;
        self
    }
}

impl WidthProperty for Slider {
    const WIDTH: f32 = 20.0;
}
impl HeightProperty for Slider {
    const HEIGHT: f32 = 1.0;
}
impl FilledProperty for Slider {
    const FILLED: char = Elements::THICK_HORIZONTAL_LINE;
}
impl UnfilledProperty for Slider {
    const UNFILLED: char = Elements::THICK_DASH_HORIZONTAL_LINE;
}

pub struct Slider<T: 'static = (), F = ()> {
    previous: f32,
    params: F,
    _marker: std::marker::PhantomData<T>,
}

impl<T: 'static, F: for<'a> FnOnce(&'a mut T) -> SliderParams<'a> + Clone> View<T>
    for Slider<T, F>
{
    type Args<'a> = F;
    type Response = SliderResponse;

    fn create(args: Self::Args<'_>) -> Self {
        Self {
            params: args,
            previous: 0.0,
            _marker: std::marker::PhantomData,
        }
    }

    fn update(&mut self, ctx: UpdateCtx<T>, args: Self::Args<'_>) -> Self::Response {
        self.params = args;
        let params = (self.params.clone())(ctx.state);

        SliderResponse {
            changed: *params.value.expect("valid state") != self.previous,
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
        let params = (self.params.clone())(ctx.state);
        let value = params.value.expect("valid state");
        self.previous = *value;

        let p = (ev.pos.x - min) / (max - min);
        *value = denormalize(p, params.min..=params.max);

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
        let params = (self.params.clone())(ctx.state);
        let value = match params.value {
            Some(value) => value,
            None => return,
        };

        // TODO axis
        let (min, max) = (ctx.rect.left(), ctx.rect.right() - 1.0);
        // we need to round to the next step
        let x = normalize(*value, params.min..=params.max);
        let x = min + (x * (max - min));

        let unfilled = ctx.properties.unfilled::<Slider>();
        let pixel = Pixel::new(unfilled).fg(ctx.theme.outline);
        ctx.surface.fill(pixel);

        let track = ctx.properties.filled::<Slider>();
        let pixel = Pixel::new(track).fg(ctx.theme.contrast);

        let pos = x - ctx.rect.left();
        ctx.surface.horizontal_fill((0.0, pos), pixel);

        let point = Point::new(pos, 0.0);
        let &Knob(knob) = ctx.properties.get_or_default::<Knob>();
        let pixel = Pixel::new(knob).fg(ctx.theme.primary);
        ctx.surface.set(point, pixel);
    }
}

pub fn slider<T: 'static>(
    ctx: &mut Context<T>,
    get: impl for<'a> FnOnce(&'a mut T) -> SliderParams<'a> + Clone + 'static,
) -> SliderResponse {
    Slider::show(get, ctx)
}
