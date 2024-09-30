use too::{math::pos2, Pixel};

use crate::{
    geom::{Size, Space},
    view::Context,
    DrawCtx, Elements, Event, EventCtx, FilledProperty, Handled, HeightProperty, Interest,
    LayoutCtx, NoResponse, Response, UnfilledProperty, UpdateCtx, View, ViewExt, WidthProperty,
};

impl WidthProperty for Toggle {
    const WIDTH: f32 = 4.0;
}
impl HeightProperty for Toggle {
    const HEIGHT: f32 = 1.0;
}
impl FilledProperty for Toggle {
    const FILLED: char = Elements::LARGE_RECT;
}
impl UnfilledProperty for Toggle {
    const UNFILLED: char = Elements::MEDIUM_RECT;
}

pub struct Toggle<T: 'static = ()> {
    args: fn(&mut T) -> &mut bool,
}

impl<T: 'static> View<T> for Toggle<T> {
    type Args<'a> = fn(&mut T) -> &mut bool;
    type Response = NoResponse;

    fn create(args: Self::Args<'_>) -> Self {
        Self { args }
    }

    fn update(&mut self, ctx: UpdateCtx<T>, args: Self::Args<'_>) -> Self::Response {
        self.args = args;
    }

    fn interest(&self) -> Interest {
        Interest::MOUSE
    }

    fn event(&mut self, ctx: EventCtx<T>, event: &Event) -> Handled {
        let Event::MouseClick(..) = event else {
            return Handled::Bubble;
        };

        let selected = (self.args)(ctx.state);
        *selected = !*selected;

        Handled::Sink
    }

    fn layout(&mut self, ctx: LayoutCtx<T>, space: Space) -> Size {
        // TODO axis unpack
        Size::new(
            ctx.properties.width::<Toggle>(),
            ctx.properties.height::<Toggle>(),
        )
    }

    fn draw(&mut self, ctx: DrawCtx<T>) {
        let selected = *((self.args)(ctx.state));
        let bg = if selected {
            ctx.theme.success
        } else {
            ctx.theme.surface
        };

        // TODO axis unpack

        let pixel = Pixel::new(ctx.properties.unfilled::<Toggle>()).fg(ctx.theme.contrast);
        ctx.surface.draw(pixel);

        let x = if selected {
            ctx.rect.width() as i32 - 1
        } else {
            0
        };

        let pixel = Pixel::new(ctx.properties.filled::<Toggle>()).fg(ctx.theme.primary);
        ctx.surface.put(pos2(x, 0), pixel);
    }
}

pub fn toggle<T: 'static>(ctx: &mut Context<T>, args: fn(&mut T) -> &mut bool) -> Response {
    Toggle::show(args, ctx)
}
