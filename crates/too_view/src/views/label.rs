use std::borrow::Cow;

use too::{shapes::Text, Attribute};

use crate::{
    geom::{Size, Space},
    view::Context,
    DrawCtx, LayoutCtx, NoResponse, Response, UpdateCtx, View, ViewExt,
};

#[derive(Clone)]
pub struct LabelParams<'a> {
    pub label: Cow<'a, str>,
    pub attribute: Option<Attribute>,
}

impl<'a> From<&'a str> for LabelParams<'static> {
    fn from(value: &'a str) -> Self {
        Self::new(value.to_string())
    }
}

impl From<String> for LabelParams<'static> {
    fn from(value: String) -> Self {
        Self::new(value)
    }
}

impl<'a> LabelParams<'a> {
    pub fn new(label: impl Into<Cow<'a, str>>) -> Self {
        Self {
            label: label.into(),
            attribute: None,
        }
    }

    pub const fn attribute(mut self, attr: Attribute) -> Self {
        self.attribute = Some(attr);
        self
    }

    pub const fn bold(self) -> Self {
        self.attribute(Attribute::BOLD)
    }

    pub const fn faint(self) -> Self {
        self.attribute(Attribute::FAINT)
    }

    pub const fn italic(self) -> Self {
        self.attribute(Attribute::ITALIC)
    }

    pub const fn underline(self) -> Self {
        self.attribute(Attribute::UNDERLINE)
    }

    pub const fn blink(self) -> Self {
        self.attribute(Attribute::BLINK)
    }

    pub const fn reverse(self) -> Self {
        self.attribute(Attribute::REVERSE)
    }

    pub const fn strikeout(self) -> Self {
        self.attribute(Attribute::STRIKEOUT)
    }
}

struct StaticLabel<T: 'static> {
    args: LabelParams<'static>,
    _marker: std::marker::PhantomData<T>,
}

impl<T: 'static> View<T> for StaticLabel<T> {
    type Args<'a> = LabelParams<'static>;
    type Response = NoResponse;

    fn create(args: Self::Args<'_>) -> Self {
        Self {
            args,
            _marker: std::marker::PhantomData,
        }
    }

    fn update(&mut self, ctx: UpdateCtx<T>, args: Self::Args<'_>) -> Self::Response {
        self.args = args
    }

    fn layout(&mut self, ctx: LayoutCtx<T>, space: Space) -> Size {
        use too::shapes::Label as _;
        self.args.label.size().into()
    }

    fn draw(&mut self, ctx: DrawCtx<T>) {
        ctx.surface.draw(
            Text::new(&self.args.label)
                .fg(ctx.theme.foreground)
                .maybe_attribute(self.args.attribute),
        );
    }
}

pub fn static_label<T: 'static>(
    ctx: &mut Context<T>,
    label: impl Into<LabelParams<'static>>,
) -> Response {
    StaticLabel::show(label.into(), ctx)
}

pub(crate) struct LabelArgs<T: 'static> {
    pub(crate) params: for<'t> fn(&'t T) -> LabelParams<'t>,
    pub(crate) attribute: Option<Attribute>,
}

impl<T: 'static> Copy for LabelArgs<T> {}

impl<T: 'static> Clone for LabelArgs<T> {
    fn clone(&self) -> Self {
        *self
    }
}

pub(crate) struct Label<T: 'static> {
    args: LabelArgs<T>,
}

impl<T: 'static> View<T> for Label<T> {
    type Args<'a> = LabelArgs<T>;
    type Response = NoResponse;

    fn create(args: Self::Args<'_>) -> Self {
        Self { args }
    }

    fn update(&mut self, ctx: UpdateCtx<T>, args: Self::Args<'_>) -> Self::Response {
        self.args = args;
    }

    fn layout(&mut self, ctx: LayoutCtx<T>, space: Space) -> Size {
        use too::shapes::Label as _;
        let params = (self.args.params)(ctx.state);
        params.label.size().into()
    }

    fn draw(&mut self, ctx: DrawCtx<T>) {
        let params = (self.args.params)(ctx.state);
        ctx.surface.draw(
            Text::new(params.label)
                .fg(ctx.theme.foreground)
                .maybe_attribute(params.attribute)
                .maybe_attribute(self.args.attribute),
        );
    }
}

pub fn label<T: 'static>(
    ctx: &mut Context<T>,
    label: for<'t> fn(&'t T) -> LabelParams<'t>,
) -> Response {
    let args = LabelArgs {
        params: label,
        attribute: None,
    };
    Label::show(args, ctx)
}
