use crate::{
    geom::{Size, Space},
    input::{Event, EventCtx, Handled},
    DrawCtx, Interest, LayoutCtx, Response, Ui, UpdateCtx,
};

pub trait Args: Clone {}
impl<T: Clone> Args for T {}

pub type NoArgs = ();
pub type NoResponse = ();

pub trait View<T: 'static>: Sized {
    type Args<'a>: Args;
    type Response;

    fn create(args: Self::Args<'_>) -> Self;

    fn update(&mut self, ctx: UpdateCtx<T>, args: Self::Args<'_>) -> Self::Response;

    fn interest(&self) -> Interest {
        Interest::NONE
    }

    fn event(&mut self, ctx: EventCtx<T>, event: &Event) -> Handled {
        _ = event;
        Handled::Bubble
    }

    fn animate(&mut self, state: &mut T, dt: f32) {
        _ = dt
    }

    fn layout(&mut self, ctx: LayoutCtx<T>, space: Space) -> Size {
        self.default_layout(ctx, space)
    }

    fn draw(&mut self, ctx: DrawCtx<T>) {
        self.default_draw(ctx);
    }

    fn default_layout(&mut self, mut ctx: LayoutCtx<T>, space: Space) -> Size {
        let mut size = Size::ZERO;
        for &child in ctx.children {
            size = size.max(ctx.compute_layout(child, space))
        }
        size
    }

    fn default_draw(&mut self, mut ctx: DrawCtx<T>) {
        for &child in ctx.children {
            ctx.draw(child)
        }
    }
}

pub struct Context<'a, T: 'static> {
    pub ui: &'a mut Ui<T>,
    pub state: &'a mut T,
}

impl<'a, T: 'static> std::ops::Deref for Context<'a, T> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        self.state
    }
}

impl<'a, T: 'static> std::ops::DerefMut for Context<'a, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.state
    }
}

pub trait ViewExt<T: 'static>: View<T> + Sized + 'static {
    fn show(args: Self::Args<'_>, ctx: &mut Context<'_, T>) -> Response<Self::Response> {
        let resp = ctx.ui.begin_view::<Self>(ctx.state, args);
        ctx.ui.end_view(resp.view_id());
        resp
    }

    fn show_children<R>(
        args: Self::Args<'_>,
        ctx: &mut Context<'_, T>,
        show: impl FnOnce(&mut Context<'_, T>) -> R,
    ) -> Response<Self::Response, R> {
        let resp = ctx.ui.begin_view::<Self>(ctx.state, args);
        let output = show(ctx);
        ctx.ui.end_view(resp.view_id());
        resp.map_output(output)
    }
}

impl<T: 'static, V: View<T> + 'static> ViewExt<T> for V {}
