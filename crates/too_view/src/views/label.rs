use std::borrow::Cow;

use too::{Attribute, Rgba};

use crate::{
    geom::{Size, Space},
    text::Text,
    view::Context,
    DrawCtx, LayoutCtx, UpdateCtx, View, ViewExt,
};

#[derive(Clone)]
pub struct LabelParams<'a> {
    pub label: Cow<'a, str>,
    pub fg: Option<Rgba>,
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
            fg: None,
            attribute: None,
        }
    }

    pub fn fg(mut self, color: impl Into<Rgba>) -> Self {
        self.fg = Some(color.into());
        self
    }

    pub fn maybe_fg(mut self, color: Option<impl Into<Rgba>>) -> Self {
        self.fg = color.map(Into::into);
        self
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

struct StaticLabel {
    args: LabelParams<'static>,
}

impl<T: 'static> View<T> for StaticLabel {
    type Args<'a> = LabelParams<'static>;
    type Response = ();

    fn create(args: Self::Args<'_>) -> Self {
        Self { args }
    }

    fn update(&mut self, ctx: UpdateCtx<T>, args: Self::Args<'_>) -> Self::Response {
        self.args = args
    }

    fn layout(&mut self, ctx: LayoutCtx<T>, space: Space) -> Size {
        Text::measure(&self.args.label)
    }

    fn draw(&mut self, mut ctx: DrawCtx<T>) {
        Text {
            data: &self.args.label,
            fg: self.args.fg.unwrap_or(ctx.theme.foreground),
            attribute: self.args.attribute.unwrap_or(Attribute::RESET),
        }
        .draw(ctx.rect, ctx.surface.surface_raw());
    }
}

pub fn static_label<T: 'static>(ctx: &mut Context<T>, label: impl Into<LabelParams<'static>>) {
    StaticLabel::show(label.into(), ctx);
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
    type Response = ();

    fn create(args: Self::Args<'_>) -> Self {
        Self { args }
    }

    fn update(&mut self, ctx: UpdateCtx<T>, args: Self::Args<'_>) -> Self::Response {
        self.args = args
    }

    fn layout(&mut self, ctx: LayoutCtx<T>, space: Space) -> Size {
        let params = (self.args.params)(ctx.state);
        Text::measure(&params.label)
    }

    fn draw(&mut self, mut ctx: DrawCtx<T>) {
        let params = (self.args.params)(ctx.state);
        Text {
            data: &params.label,
            fg: ctx.theme.foreground,
            attribute: self.args.attribute.unwrap_or(Attribute::RESET),
        }
        .draw(ctx.rect, ctx.surface.surface_raw());
    }
}

pub fn label<T: 'static>(ctx: &mut Context<T>, label: for<'t> fn(&'t T) -> LabelParams<'t>) {
    let args = LabelArgs {
        params: label,
        attribute: None,
    };
    Label::show(args, ctx);
}
