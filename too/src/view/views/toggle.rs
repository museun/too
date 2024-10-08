use std::{borrow::Cow, rc::Rc, time::Duration};

use crate::{
    animation::{easing, Animation},
    math::lerp,
    Pixel,
};

use super::super::{
    elements::Elements,
    geom::{Size, Space},
    properties::*,
    view::Context,
    DrawCtx, Event, EventCtx, Handled, Interest, LayoutCtx, UpdateCtx, View, ViewExt,
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

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct ToggleResponse {
    pub changed: bool,
}

pub struct Toggle<T: 'static = (), F = ()> {
    args: F,
    last: bool,
    _marker: std::marker::PhantomData<T>,
}

impl<T: 'static, F: for<'t> FnOnce(&'t mut T) -> ToggleParams<'t> + Clone> View<T>
    for Toggle<T, F>
{
    type Args<'a> = F;
    type Response = ToggleResponse;

    fn create(args: Self::Args<'_>) -> Self {
        Self {
            args,
            last: false,
            _marker: std::marker::PhantomData,
        }
    }

    fn short_name() -> std::borrow::Cow<'static, str> {
        Cow::from("Toggle")
    }

    fn update(&mut self, ctx: UpdateCtx<T>, args: Self::Args<'_>) -> Self::Response {
        self.args = args;
        let value = (self.args.clone())(ctx.state);
        let bool = match value.flag {
            ToggleBool::Bool(bool) => *bool,
            ToggleBool::Shared(shared) => shared.get(),
        };

        let previous = std::mem::replace(&mut self.last, bool);
        ToggleResponse {
            changed: previous != self.last,
        }
    }

    fn interest(&self) -> Interest {
        Interest::MOUSE
    }

    fn event(&mut self, ctx: EventCtx<T>, event: &Event) -> Handled {
        let Event::MouseClick(..) = event else {
            return Handled::Bubble;
        };

        let selected = (self.args.clone())(ctx.state);
        match selected.flag {
            ToggleBool::Bool(bool) => *bool = !*bool,
            ToggleBool::Shared(shared) => {
                let temp = shared.get();
                shared.set(!temp);
            }
        };

        let key = ctx.current_id;
        ctx.animations.add(
            key,
            Animation::new()
                .with(easing::sine_in_out)
                .oneshot(true)
                .schedule(Duration::from_millis(150))
                .unwrap(),
            0.0,
        );

        Handled::Sink
    }

    fn layout(&mut self, ctx: LayoutCtx<T>, space: Space) -> Size {
        // TODO axis unpack
        Size::new(
            ctx.properties.width::<Toggle>(),
            ctx.properties.height::<Toggle>(),
        )
    }

    fn draw(&mut self, mut ctx: DrawCtx<T>) {
        let selected = (self.args.clone())(ctx.state);
        let selected = match selected.flag {
            ToggleBool::Bool(bool) => *bool,
            ToggleBool::Shared(shared) => shared.get(),
        };

        let bg = if selected {
            ctx.theme.success
        } else {
            ctx.theme.surface
        };

        // TODO axis unpack

        let unfilled = ctx.properties.unfilled::<Toggle>();
        let pixel = Pixel::new(unfilled).fg(ctx.theme.contrast);
        ctx.surface.fill(pixel);

        // this - 1.0 is because of the knob size
        let width_inclusive = ctx.rect.width() - 1.0;

        let view_id = ctx.current_id;
        let x = match ctx.animations.get_mut(view_id) {
            Some(animation) if selected => lerp(0.0, width_inclusive, *animation.value),
            Some(animation) if !selected => lerp(width_inclusive, 0.0, *animation.value),
            _ if selected => width_inclusive,
            _ => 0.0,
        };

        let filled = ctx.properties.filled::<Toggle>();
        let pixel = Pixel::new(filled).fg(ctx.theme.primary);
        ctx.surface.set((x, 0.0), pixel);
    }
}

pub struct ToggleParams<'a> {
    flag: ToggleBool<'a>,
}

impl<'a> ToggleParams<'a> {
    pub fn new(bool: &'a mut bool) -> Self {
        Self {
            flag: ToggleBool::Bool(bool),
        }
    }

    pub fn shared(shared: Rc<std::cell::Cell<bool>>) -> Self {
        Self {
            flag: ToggleBool::Shared(shared),
        }
    }
}

enum ToggleBool<'a> {
    Bool(&'a mut bool),
    Shared(Rc<std::cell::Cell<bool>>),
}

// TODO this should take in a closure
pub fn toggle<T: 'static>(
    ctx: &mut Context<T>,
    args: impl for<'a> FnOnce(&'a mut T) -> ToggleParams<'a> + Clone + 'static,
) -> ToggleResponse {
    Toggle::show(args, ctx)
}
