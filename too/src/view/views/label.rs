use std::borrow::Cow;

use crate::{Attribute, Color, Rgba};

use super::super::{
    geom::{Size, Space},
    text::Text,
    view::Context,
    DrawCtx, LayoutCtx, UpdateCtx, View, ViewExt,
};

#[derive(Clone)]
pub struct LabelParams<'a> {
    pub inner: Option<LabelOptions<'a>>,
}

impl<'a> From<&'a str> for LabelParams<'static> {
    fn from(value: &'a str) -> Self {
        Self::new(LabelOptions {
            label: Cow::from(value.to_string()),
            fg: None,
            attribute: None,
        })
    }
}

impl From<String> for LabelParams<'static> {
    fn from(value: String) -> Self {
        Self::new(LabelOptions {
            label: Cow::from(value),
            fg: None,
            attribute: None,
        })
    }
}

#[derive(Clone)]
pub struct LabelOptions<'a> {
    label: Cow<'a, str>,
    fg: Option<Rgba>,
    attribute: Option<Attribute>,
}

impl<'a> LabelOptions<'a> {
    pub fn new(s: impl Into<Cow<'a, str>>) -> Self {
        Self {
            label: s.into(),
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

impl<'a> From<&'a str> for LabelOptions<'static> {
    fn from(value: &'a str) -> Self {
        LabelOptions {
            label: Cow::from(value.to_string()),
            fg: None,
            attribute: None,
        }
    }
}

impl From<String> for LabelOptions<'static> {
    fn from(value: String) -> Self {
        LabelOptions {
            label: Cow::from(value),
            fg: None,
            attribute: None,
        }
    }
}

impl<'a> LabelParams<'a> {
    // this `C` is very unfortunate
    pub fn new(label: impl Into<Option<LabelOptions<'a>>>) -> Self {
        Self {
            inner: label.into(),
        }
    }

    pub fn fg(mut self, color: impl Into<Rgba>) -> Self {
        if let Some(inner) = self.inner.as_mut() {
            inner.fg = Some(color.into());
        }
        self
    }

    pub fn maybe_fg(mut self, color: Option<impl Into<Rgba>>) -> Self {
        if let Some(inner) = self.inner.as_mut() {
            inner.fg = color.map(Into::into)
        }
        self
    }

    pub fn attribute(mut self, attr: Attribute) -> Self {
        if let Some(inner) = self.inner.as_mut() {
            inner.attribute = Some(attr)
        }
        self
    }

    pub fn bold(self) -> Self {
        self.attribute(Attribute::BOLD)
    }

    pub fn faint(self) -> Self {
        self.attribute(Attribute::FAINT)
    }

    pub fn italic(self) -> Self {
        self.attribute(Attribute::ITALIC)
    }

    pub fn underline(self) -> Self {
        self.attribute(Attribute::UNDERLINE)
    }

    pub fn blink(self) -> Self {
        self.attribute(Attribute::BLINK)
    }

    pub fn reverse(self) -> Self {
        self.attribute(Attribute::REVERSE)
    }

    pub fn strikeout(self) -> Self {
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
        let Some(label) = self.args.inner.as_ref() else {
            return Size::ZERO;
        };

        Text::measure(&label.label)
    }

    fn draw(&mut self, mut ctx: DrawCtx<T>) {
        if let Some(inner) = self.args.inner.as_ref() {
            Text {
                data: &inner.label,
                fg: inner.fg.unwrap_or(ctx.theme.foreground),
                bg: Color::Reuse,
                // bg: Color::Set(ctx.theme.background),
                attribute: inner.attribute.unwrap_or(Attribute::RESET),
            }
            .draw(ctx.rect, ctx.surface.surface_raw());
        }
    }
}

pub fn static_label<T: 'static>(ctx: &mut Context<T>, label: impl Into<LabelParams<'static>>) {
    StaticLabel::show(label.into(), ctx);
}

pub(crate) struct LabelArgs<T, F>
where
    T: 'static,
    F: for<'a> FnOnce(&'a T) -> LabelParams<'a> + Clone,
{
    pub(crate) params: F,
    pub(crate) _marker: std::marker::PhantomData<T>,
}

impl<T, F> Clone for LabelArgs<T, F>
where
    T: 'static,
    F: for<'a> FnOnce(&'a T) -> LabelParams<'a> + Clone,
{
    fn clone(&self) -> Self {
        Self {
            params: self.params.clone(),
            _marker: std::marker::PhantomData,
        }
    }
}

pub(crate) struct Label<T, F>
where
    T: 'static,
    F: for<'a> FnOnce(&'a T) -> LabelParams<'a> + Clone,
{
    args: LabelArgs<T, F>,
}

impl<T, F> View<T> for Label<T, F>
where
    T: 'static,
    F: for<'a> FnOnce(&'a T) -> LabelParams<'a> + Clone,
{
    type Args<'a> = LabelArgs<T, F>;
    type Response = ();

    fn create(args: Self::Args<'_>) -> Self {
        Self { args }
    }

    fn short_name() -> Cow<'static, str> {
        Cow::from("Label")
    }

    fn update(&mut self, ctx: UpdateCtx<T>, args: Self::Args<'_>) -> Self::Response {
        self.args = args
    }

    fn layout(&mut self, ctx: LayoutCtx<T>, space: Space) -> Size {
        let params = (self.args.params.clone())(ctx.state);
        let Some(inner) = params.inner.as_ref() else {
            return Size::ZERO;
        };
        Text::measure(&inner.label)
    }

    fn draw(&mut self, mut ctx: DrawCtx<T>) {
        let params = (self.args.params.clone())(ctx.state);
        if let Some(inner) = params.inner.as_ref() {
            Text {
                data: &inner.label,
                fg: inner.fg.unwrap_or(ctx.theme.foreground),
                bg: Color::Set(ctx.theme.background),
                attribute: inner.attribute.unwrap_or(Attribute::RESET),
            }
            .draw(ctx.rect, ctx.surface.surface_raw());
        }
    }
}

pub fn label<T: 'static>(
    ctx: &mut Context<T>,
    label: impl for<'t> FnOnce(&'t T) -> LabelParams<'t> + Clone + 'static,
) {
    let args = LabelArgs {
        params: label,
        _marker: std::marker::PhantomData,
    };
    Label::show(args, ctx);
}
