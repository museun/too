use crate::{view::geom::Margin, Attribute, Color};

use super::super::{
    geom::{Size, Space, Vector},
    text::Text,
    view::Context,
    DrawCtx, Event, EventCtx, Handled, Interest, LayoutCtx, UpdateCtx, View, ViewExt,
};

use super::{
    background::background,
    label::{label, static_label, LabelParams},
    list::{list, List},
    mouse_area::on_click,
};

pub struct ButtonParams<'a> {
    pub label: &'a str,
    pub enabled: bool,
    pub margin: Margin,
}

impl<'a> ButtonParams<'a> {
    pub const fn new(label: &'a str) -> Self {
        Self {
            label,
            enabled: true,
            margin: Margin::same(0.0),
        }
    }

    pub fn margin(mut self, margin: impl Into<Margin>) -> Self {
        self.margin = margin.into();
        self
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
        self.params = args;
        match self.state {
            ButtonState::Clicked => {
                self.state = ButtonState::Hovered;
                true
            }
            _ => false,
        }
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

        space.fit(Text::measure(params.label) + params.margin.sum()) + Size::new(2.0, 0.0)
    }

    fn draw(&mut self, mut ctx: DrawCtx<T>) {
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

        let offset = ctx.rect + Vector::X;
        ctx.surface.fill(bg);

        Text {
            data: params.label,
            fg,
            bg: Color::Reuse,
            attribute: Attribute::RESET,
        }
        .draw(params.margin.shrink_rect(offset), ctx.surface.surface_raw());
    }
}

pub fn button<T: 'static>(ctx: &mut Context<T>, params: fn(&T) -> ButtonParams<'_>) -> bool {
    Button::show(params, ctx)
}

pub fn checkbox<T: 'static>(
    ctx: &mut Context<T>,
    value: fn(&mut T) -> &mut bool,
    text: for<'t> fn(&'t T) -> LabelParams<'t>,
) -> bool {
    let resp = on_click(ctx, move |ctx| {
        // TODO mouse over
        list(List::horizontal().gap(1.0), ctx, move |ctx| {
            let value = *(value)(ctx.state);
            let element = match value {
                true => "☒",
                false => "☐",
            };
            static_label(ctx, element);
            label(ctx, text);
        });
    });

    *(value)(ctx) ^= resp;
    resp
}

pub fn todo_value<T: 'static>(
    ctx: &mut Context<T>,
    value: fn(&mut T) -> &mut bool,
    // why?
    text: for<'t> fn(&'t T) -> LabelParams<'t>,
) -> bool {
    todo!();
    // let resp = on_click(ctx, move |ctx| {
    //     // TODO mouse over
    //     list(List::horizontal().gap(1.0), ctx, move |ctx| {
    //         let value = *(value)(ctx.state);
    //         let attr = value.then(|| Attribute::STRIKEOUT | Attribute::FAINT);

    //         let args = LabelArgs {
    //             params: text,
    //             attribute: attr,
    //             _marker: std::marker::PhantomData,
    //         };
    //         Label::show(args, ctx);
    //     });
    // });

    // *(value)(ctx) ^= resp;
    // resp
}

pub fn selected<T: 'static, R>(
    ctx: &mut Context<T>,
    value: fn(&mut T) -> &mut bool,
    show: impl FnOnce(&mut Context<T>) -> R,
) -> bool {
    let resp = on_click(ctx, move |ctx| {
        let value = (value)(ctx.state);
        let bg = if *value {
            ctx.ui.theme.primary
        } else {
            ctx.ui.theme.outline
        };
        background(ctx, bg, show)
    });

    *(value)(ctx) ^= resp;
    resp
}

pub fn radio<T: 'static, R, V: PartialEq>(
    ctx: &mut Context<T>,
    selected: V,
    value: fn(&mut T) -> &mut V,
    show: impl FnOnce(&mut Context<T>) -> R,
) -> bool {
    let resp = on_click(ctx, |ctx| {
        let val = value(ctx.state);
        let bg = if *val == selected {
            ctx.ui.theme.primary
        } else {
            ctx.ui.theme.outline
        };
        // TODO margin needs to be here
        // TODO this shouldn't be here.
        background(ctx, bg, show)
    });

    if resp {
        *value(ctx.state) = selected;
    }
    resp
}
