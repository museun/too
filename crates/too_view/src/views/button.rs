use too::{
    shapes::{Fill, Text},
    vec2, Attribute,
};

use crate::{
    geom::{Size, Space, Vector},
    response::UserResponse,
    view::Context,
    DrawCtx, Event, EventCtx, Handled, Interest, LayoutCtx, Response, UpdateCtx, View, ViewExt,
};

use super::{
    background::background,
    label::{label, static_label, Label, LabelArgs, LabelParams},
    list::{list, ListParams},
    mouse_area::on_click,
};

pub struct ButtonParams<'a> {
    pub label: &'a str,
    pub enabled: bool,
}

impl<'a> ButtonParams<'a> {
    pub const fn new(label: &'a str) -> Self {
        Self {
            label,
            enabled: true,
        }
    }

    pub const fn enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }
}

#[derive(Copy, Clone, Default)]
enum ButtonState {
    Hovered,
    Held,
    Clicked,
    #[default]
    None,
}

struct Button<T: 'static> {
    params: fn(&T) -> ButtonParams<'_>,
    state: ButtonState,
}

impl<T: 'static> View<T> for Button<T> {
    type Args<'a> = fn(&T) -> ButtonParams<'_>;
    type Response = bool;

    fn create(args: Self::Args<'_>) -> Self {
        Self {
            params: args,
            state: ButtonState::None,
        }
    }

    fn update(&mut self, ctx: UpdateCtx<T>, args: Self::Args<'_>) -> Self::Response {
        let clicked = match self.state {
            ButtonState::Clicked => {
                self.state = ButtonState::Hovered;
                true
            }
            _ => false,
        };

        self.params = args;
        clicked
    }

    fn event(&mut self, ctx: EventCtx<T>, event: &Event) -> Handled {
        if !(self.params)(ctx.state).enabled {
            return Handled::Bubble;
        }

        // TODO answer 'which button'
        self.state = match event {
            Event::MouseEnter(..) => ButtonState::Hovered,
            Event::MouseLeave(..) => ButtonState::None,
            Event::MouseClick(..) => ButtonState::Clicked,
            Event::MouseHeld(..) => ButtonState::Held,
            _ => return Handled::Bubble,
        };

        Handled::Sink
    }

    fn interest(&self) -> Interest {
        Interest::MOUSE
    }

    fn layout(&mut self, ctx: LayoutCtx<T>, space: Space) -> Size {
        let params = (self.params)(ctx.state);
        let size = Text::new(params.label).size();
        Size::from(size) + Vector::new(2.0, 0.0)
    }

    fn draw(&mut self, ctx: DrawCtx<T>) {
        let params = (self.params)(ctx.state);

        let fg = if params.enabled {
            ctx.theme.foreground
        } else {
            ctx.theme.outline
        };

        let bg = match self.state {
            ButtonState::Hovered if params.enabled => ctx.theme.accent,
            ButtonState::Held if params.enabled => ctx.theme.primary,
            ButtonState::Clicked if params.enabled => ctx.theme.success,
            ButtonState::None if params.enabled => ctx.theme.surface,
            _ => ctx.theme.surface,
        };

        let offset = ctx.surface.rect().translate(vec2(1, 0));
        ctx.surface
            .draw(Fill::new(bg))
            .crop(offset) // this is such a hack
            .draw(Text::new(params.label).fg(fg));
    }
}

pub fn button<T: 'static>(
    ctx: &mut Context<'_, T>,
    params: fn(&T) -> ButtonParams<'_>,
) -> Response<bool> {
    Button::show(params, ctx)
}

pub fn checkbox<T: 'static>(
    ctx: &mut Context<'_, T>,
    value: fn(&mut T) -> &mut bool,
    text: for<'t> fn(&'t T) -> LabelParams<'t>,
) -> Response<bool> {
    let resp = on_click(ctx, move |ctx| {
        // TODO mouse over
        list(ListParams::horizontal().gap(1.0), ctx, move |ctx| {
            let value = *(value)(ctx.state);
            let element = match value {
                true => "☒",
                false => "☐",
            };
            static_label(ctx, element);
            label(ctx, text);
        });
    });

    *(value)(ctx) ^= *resp;
    resp
}

pub fn todo_value<T: 'static>(
    ctx: &mut Context<'_, T>,
    value: fn(&mut T) -> &mut bool,
    text: for<'t> fn(&'t T) -> LabelParams<'t>,
) -> Response<bool> {
    let resp = on_click(ctx, move |ctx| {
        // TODO mouse over
        list(ListParams::horizontal().gap(1.0), ctx, move |ctx| {
            let value = *(value)(ctx.state);
            let attr = value.then(|| Attribute::STRIKEOUT | Attribute::FAINT);
            let args = LabelArgs {
                params: text,
                attribute: None,
            };
            Label::show(args, ctx);
        });
    });

    *(value)(ctx) ^= *resp;
    resp
}

pub fn selected<T: 'static, R>(
    ctx: &mut Context<'_, T>,
    value: fn(&mut T) -> &mut bool,
    show: impl FnOnce(&mut Context<T>) -> R,
) -> Response<bool, R> {
    let resp = on_click(ctx, move |ctx| {
        let value = (value)(ctx.state);
        let bg = if *value {
            ctx.ui.theme.primary
        } else {
            ctx.ui.theme.outline
        };
        background(bg, ctx, show)
    });

    let id = resp.view_id();
    let okay = *resp;
    // TODO why 2 into_inners
    let inner = resp.into_inner().into_inner();
    *(value)(ctx) ^= okay;
    Response::new(id, okay, inner)
}

pub fn radio<T: 'static, R, V: PartialEq>(
    ctx: &mut Context<'_, T>,
    selected: V,
    value: fn(&mut T) -> &mut V,
    show: impl FnOnce(&mut Context<'_, T>) -> R,
) -> UserResponse<R> {
    let resp = on_click(ctx, |ctx| {
        let val = value(ctx.state);
        let bg = if *val == selected {
            ctx.ui.theme.primary
        } else {
            ctx.ui.theme.outline
        };
        // margin needs to be here
        background(bg, ctx, show)
    });

    if *resp {
        *value(ctx.state) = selected;
    }

    resp.into_inner()
}
