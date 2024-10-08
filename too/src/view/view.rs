use std::{any::type_name, borrow::Cow};

use super::{
    debug_fmt::short_name,
    geom::{Size, Space},
    input::{Event, EventCtx, Handled},
    AnimateCtx, DestroyCtx, DrawCtx, Interest, LayoutCtx, Ui, UpdateCtx,
};

pub trait Args: Clone {}
impl<T: Clone> Args for T {}

pub trait View<T: 'static>: Sized {
    type Args<'a>: Args;
    type Response;

    fn create(args: Self::Args<'_>) -> Self;

    fn short_name() -> Cow<'static, str> {
        Self::default_short_name()
    }

    fn update(&mut self, ctx: UpdateCtx<T>, args: Self::Args<'_>) -> Self::Response;

    fn interest(&self) -> Interest {
        Interest::NONE
    }

    fn event(&mut self, ctx: EventCtx<T>, event: &Event) -> Handled {
        _ = event;
        Handled::Bubble
    }

    fn animate(&mut self, ctx: AnimateCtx<T>, dt: f32) {
        self.default_animate(ctx, dt);
    }

    fn layout(&mut self, ctx: LayoutCtx<T>, space: Space) -> Size {
        self.default_layout(ctx, space)
    }

    fn draw(&mut self, ctx: DrawCtx<T>) {
        self.default_draw(ctx);
    }

    fn destroy(&mut self, ctx: DestroyCtx<T>) {
        self.default_destroy(ctx)
    }

    fn default_short_name() -> Cow<'static, str> {
        short_name(type_name::<Self>()).into()
    }

    fn default_animate(&mut self, mut ctx: AnimateCtx<T>, dt: f32) {
        for &child in ctx.children {
            ctx.animate(child, dt);
        }
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

    fn default_destroy(&mut self, mut ctx: DestroyCtx<T>) {
        for &child in ctx.children {
            ctx.destroy(child)
        }
    }
}

pub struct Context<'a, 'b, T: 'static> {
    pub ui: &'a mut Ui<T>,
    pub state: &'b mut T,
}

impl<'a, 'b, T: 'static> std::ops::Deref for Context<'a, 'b, T> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        self.state
    }
}

impl<'a, 'b, T: 'static> std::ops::DerefMut for Context<'a, 'b, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.state
    }
}

pub trait ViewExt<T: 'static>: View<T> + Sized + 'static {
    fn show(args: Self::Args<'_>, ctx: &mut Context<T>) -> Self::Response {
        let (id, resp) = ctx.ui.begin_view::<Self>(ctx.state, args);
        ctx.ui.end_view(id);
        resp
    }

    fn show_children<R>(
        args: Self::Args<'_>,
        ctx: &mut Context<T>,
        show: impl FnOnce(&mut Context<T>) -> R,
    ) -> (Self::Response, R) {
        let (id, resp) = ctx.ui.begin_view::<Self>(ctx.state, args);
        let inner = show(ctx);
        ctx.ui.end_view(id);
        (resp, inner)
    }
}

impl<T: 'static, V: View<T> + 'static> ViewExt<T> for V {}
