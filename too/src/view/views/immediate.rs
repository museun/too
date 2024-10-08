use too::CroppedSurface;

use crate::{
    geom::Space, view::Context, AnimateCtx, DrawCtx, EventCtx, Handled, Interest, LayoutCtx,
    UpdateCtx, View, ViewExt,
};

struct Immediate<T, A>
where
    T: 'static,
    A: too::App,
{
    app: fn(&mut T) -> &mut A,
}

impl<T: 'static, A> View<T> for Immediate<T, A>
where
    T: 'static,
    A: too::App,
{
    type Args<'a> = fn(&mut T) -> &mut A;
    type Response = ();

    fn create(args: Self::Args<'_>) -> Self {
        Self { app: args }
    }

    fn update(&mut self, ctx: UpdateCtx<T>, args: Self::Args<'_>) -> Self::Response {
        self.app = args
    }

    fn interest(&self) -> Interest {
        Interest::MOUSE | Interest::KEY_INPUT
    }

    fn event(&mut self, mut ctx: EventCtx<T>, event: &crate::Event) -> Handled {
        let Some(ev) = event.translate_to_too() else {
            return Handled::Bubble;
        };

        let app = (self.app)(ctx.state);
        ctx.runner.size = ctx.rect.size().into();
        app.event(ev, ctx.runner);
        Handled::Sink
    }

    fn layout(&mut self, ctx: LayoutCtx<T>, space: Space) -> crate::geom::Size {
        space.max
    }

    fn animate(&mut self, ctx: AnimateCtx<T>, dt: f32) {
        let app = (self.app)(ctx.state);
        app.update(dt, ctx.runner);
    }

    fn draw(&mut self, mut ctx: DrawCtx<T>) {
        let app = (self.app)(ctx.state);
        ctx.runner.size = ctx.rect.size().into();
        let mut surface = CroppedSurface::new(ctx.rect.into(), ctx.surface.surface_raw());
        app.render(&mut surface, ctx.runner);
    }
}

pub fn immediate<T, A>(ctx: &mut Context<T>, get_app: fn(&mut T) -> &mut A)
where
    T: 'static,
    A: too::App + 'static,
{
    Immediate::show(get_app, ctx);
}
