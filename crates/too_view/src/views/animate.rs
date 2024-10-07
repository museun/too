use too::animation::AnimationManager;

use crate::{view::Context, AnimateCtx, UpdateCtx, View, ViewExt};

struct Animate<T: 'static> {
    update: fn(&mut T, f32, &mut AnimationManager),
}

impl<T: 'static> View<T> for Animate<T> {
    type Args<'a> = fn(&mut T, f32, &mut AnimationManager);
    type Response = ();

    fn create(args: Self::Args<'_>) -> Self {
        Self { update: args }
    }

    fn update(&mut self, ctx: UpdateCtx<T>, args: Self::Args<'_>) -> Self::Response {
        self.update = args
    }

    fn animate(&mut self, mut ctx: AnimateCtx<T>, dt: f32) {
        (self.update)(ctx.state, dt, ctx.too_ctx.animations_mut())
    }
}

pub fn animate<T: 'static>(ctx: &mut Context<T>, update: fn(&mut T, f32, &mut AnimationManager)) {
    Animate::show(update, ctx);
}
