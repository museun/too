use std::ops::RangeInclusive;

use too_events::{Key, Keybind, Modifiers};
use too_renderer::{Pixel, Rgba};
use too_runner::{layout::Align2, shapes::Text};
use too_shapes::Fill;

use crate::{
    geom::{self, Point, Size, Space},
    response::UserResponse,
    view::Context,
    DrawCtx, Event, EventCtx, Handled, Interest, LayoutCtx, NoArgs, NoResponse, Response, View,
    ViewExt,
};

pub(crate) struct RootView;
impl<T: 'static> View<T> for RootView {
    type Args<'a> = NoArgs;
    type Response = NoResponse;

    fn create(args: Self::Args<'_>) -> Self {
        Self
    }

    fn update(&mut self, state: &mut T, args: Self::Args<'_>) {}

    fn layout(&mut self, mut ctx: LayoutCtx<T>, space: Space) -> Size {
        ctx.new_layer();
        for &child in ctx.children {
            ctx.compute_layout(child, space);
        }
        space.max
    }
}

struct Aligned {
    align2: Align2,
}

impl<T: 'static> View<T> for Aligned {
    type Args<'a> = Align2;
    type Response = NoResponse;

    fn create(args: Self::Args<'_>) -> Self {
        Self { align2: args }
    }

    fn update(&mut self, state: &mut T, args: Self::Args<'_>) -> Self::Response {
        self.align2 = args;
    }

    fn layout(&mut self, mut ctx: LayoutCtx<T>, space: Space) -> Size {
        let space = space.loosen();

        let mut size = space.size();
        for &child in ctx.children {
            let next = ctx.compute_layout(child, space);
            size = size.max(next);
            let pos = size * self.align2 - next * self.align2;
            ctx.set_position(child, pos);
        }

        size.max(space.min.finite_or_zero())
            .max(space.max.finite_or_zero())
    }
}

pub fn center<T: 'static, R>(
    ctx: &mut Context<'_, T>,
    show: impl FnOnce(&mut Context<'_, T>) -> R,
) -> UserResponse<R> {
    align(Align2::CENTER_CENTER, ctx, show)
}

pub fn align<T: 'static, R>(
    align: Align2,
    ctx: &mut Context<'_, T>,
    show: impl FnOnce(&mut Context<'_, T>) -> R,
) -> UserResponse<R> {
    Aligned::show_children(align, ctx, show)
}

struct Label<T: 'static> {
    args: fn(&T) -> &str,
}

impl<T: 'static> View<T> for Label<T> {
    type Args<'a> = fn(&T) -> &str;
    type Response = NoResponse;

    fn create(args: Self::Args<'_>) -> Self {
        Self { args }
    }

    fn update(&mut self, state: &mut T, args: Self::Args<'_>) -> Self::Response {
        self.args = args
    }

    fn layout(&mut self, ctx: LayoutCtx<T>, space: Space) -> Size {
        use too_shapes::Label as _;
        let label = (self.args)(ctx.state);
        label.size().into()
    }

    fn draw(&mut self, ctx: DrawCtx<T>) {
        let label = (self.args)(ctx.state);
        ctx.surface.draw(Text::new(label));
    }
}

pub fn label<T: 'static>(ctx: &mut Context<'_, T>, label: fn(&T) -> &str) -> Response {
    Label::show(label, ctx)
}

struct StaticLabel<L>
where
    L: too_shapes::Label + 'static + Clone,
{
    label: L,
}

impl<L, T> View<T> for StaticLabel<L>
where
    L: too_shapes::Label + 'static + Clone,
    T: 'static,
{
    type Args<'a> = L;
    type Response = NoResponse;

    fn create(args: Self::Args<'_>) -> Self {
        Self { label: args }
    }

    fn update(&mut self, state: &mut T, args: Self::Args<'_>) {
        self.label = args;
    }

    fn layout(&mut self, ctx: LayoutCtx<T>, space: Space) -> Size {
        self.label.size().into()
    }

    fn draw(&mut self, ctx: DrawCtx<T>) {
        ctx.surface.draw(Text::new(&self.label));
    }
}

pub fn static_label<T: 'static>(
    label: impl too_shapes::Label + 'static + Clone,
    ctx: &mut Context<'_, T>,
) -> Response {
    StaticLabel::show(label, ctx)
}

struct Offset<T: 'static> {
    args: fn(&T) -> Point,
}

impl<T: 'static> View<T> for Offset<T> {
    type Args<'a> = fn(&T) -> Point;
    type Response = NoResponse;

    fn create(args: Self::Args<'_>) -> Self {
        Self { args }
    }

    fn update(&mut self, state: &mut T, args: Self::Args<'_>) -> Self::Response {
        self.args = args;
    }

    fn layout(&mut self, mut ctx: LayoutCtx<T>, space: Space) -> Size {
        let mut size = Size::ZERO;
        let offset = (self.args)(ctx.state);
        for &child in ctx.children {
            size = size.max(ctx.compute_layout(child, space));
            ctx.set_position(child, offset);
        }
        size
    }
}

pub fn offset<T: 'static, R>(
    pos: fn(&T) -> Point,
    ctx: &mut Context<'_, T>,
    show: fn(&mut Context<'_, T>) -> R,
) -> UserResponse<R> {
    Offset::show_children(pos, ctx, show)
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct KeyAreaResponse {
    // TODO maybe a `Keybind`
    pub key: Option<Key>,
    pub modifiers: Option<Modifiers>,
}

impl KeyAreaResponse {
    pub fn is_keybind(&self, keybind: impl Into<Keybind>) -> bool {
        let Some(key) = self.key else {
            return false;
        };

        let modifiers = self.modifiers.unwrap_or_default();

        let expected: Keybind = keybind.into();
        if matches!(expected.key, Key::Char(..))
            && expected.modifiers.is_none()
            && modifiers.is_none()
        {
            return key == expected.key;
        }
        Keybind::new(key, modifiers) == expected
    }
}

#[derive(Debug)]
struct KeyArea {
    key: Option<Key>,
    modifiers: Option<Modifiers>,
}

impl<T: 'static> View<T> for KeyArea {
    type Args<'a> = NoArgs;
    type Response = KeyAreaResponse;

    fn create(args: Self::Args<'_>) -> Self {
        Self {
            key: None,
            modifiers: None,
        }
    }

    fn interest(&self) -> Interest {
        Interest::KEY_INPUT
    }

    fn update(&mut self, state: &mut T, args: Self::Args<'_>) -> Self::Response {
        Self::Response {
            key: std::mem::take(&mut self.key),
            modifiers: std::mem::take(&mut self.modifiers),
        }
    }

    fn event(&mut self, ctx: EventCtx<T>, event: &Event) -> Handled {
        if let Event::KeyInput(ev) = event {
            self.key = Some(ev.key);
            self.modifiers = Some(ev.modifiers);
        }
        Handled::Bubble
    }
}

pub fn key_area<T: 'static, R>(
    ctx: &mut Context<'_, T>,
    show: impl FnOnce(&mut Context<'_, T>) -> R,
) -> Response<KeyAreaResponse, R> {
    KeyArea::show_children((), ctx, show)
}

pub fn hot_key<T: 'static, R>(
    keybind: impl Into<Keybind>,
    ctx: &mut Context<'_, T>,
    show: impl FnOnce(&mut Context<'_, T>) -> R,
) -> Response<bool, R> {
    let resp = key_area(ctx, show);
    let pressed = resp.is_keybind(keybind);
    Response::new(resp.view_id(), pressed, resp.into_inner())
}

pub fn key_press<const N: usize, T: 'static, R>(
    keys: [Keybind; N],
    ctx: &mut Context<'_, T>,
    show: impl FnOnce(&mut Context<'_, T>) -> R,
) -> Response<[bool; N], R> {
    let resp = key_area(ctx, show);
    let mut out = [false; N];
    for (key, result) in keys.into_iter().zip(&mut out) {
        *result = resp.is_keybind(key)
    }
    Response::new(resp.view_id(), out, resp.into_inner())
}

pub struct SliderParams<'a> {
    pub value: &'a mut f32,
    pub range: RangeInclusive<f32>,
}

impl<'a> SliderParams<'a> {
    pub fn new(value: &'a mut f32) -> Self {
        Self {
            value,
            range: 0.0..=1.0,
        }
    }

    pub const fn range(mut self, range: RangeInclusive<f32>) -> Self {
        self.range = range;
        self
    }
}

struct Slider<T: 'static> {
    params: fn(&mut T) -> SliderParams<'_>,
}

impl<T: 'static> Slider<T> {
    fn normalize(value: f32, range: &RangeInclusive<f32>) -> f32 {
        let value = value.clamp(*range.start(), *range.end());
        (value - range.start()) / (range.end() - range.start())
    }

    fn denormalize(value: f32, range: &RangeInclusive<f32>) -> f32 {
        let value = value.clamp(0.0, 1.0);
        value * (range.end() - range.start()) + range.start()
    }
}

impl<T: 'static> View<T> for Slider<T> {
    type Args<'a> = fn(&mut T) -> SliderParams<'_>;
    type Response = NoResponse;

    fn create(args: Self::Args<'_>) -> Self {
        Self { params: args }
    }

    fn update(&mut self, state: &mut T, args: Self::Args<'_>) -> Self::Response {}

    fn interest(&self) -> Interest {
        Interest::MOUSE
    }

    fn event(&mut self, ctx: EventCtx<T>, event: &Event) -> Handled {
        let Event::MouseDrag(ev) = event else {
            return Handled::Bubble;
        };

        let (min, max) = (ctx.rect.left(), ctx.rect.right());
        // TODO axis

        let params = (self.params)(ctx.state);
        let p = (ev.pos.x - min) / (max - min);
        *params.value = Self::denormalize(p, &params.range);

        Handled::Sink
    }

    fn layout(&mut self, ctx: LayoutCtx<T>, space: Space) -> Size {
        // TODO axis
        space.fit(Size::new(20.0, 1.0))
    }

    fn draw(&mut self, ctx: DrawCtx<T>) {
        let params = (self.params)(ctx.state);

        // TODO axis
        let (min, max) = (ctx.rect.left(), ctx.rect.right() - 1.0);
        let x = Self::normalize(*params.value, &params.range);
        let x = min + (x * (max - min));

        ctx.surface.draw(Fill::new("#555"));

        // surface::crop does not work -- we need to normalize our rect to 0,0
        for x in 0..(x - ctx.rect.left()).round() as i32 {
            ctx.surface
                .put(too_math::pos2(x, 0), Pixel::new(' ').bg("#FFF"));
        }

        let point = Point::new(x - ctx.rect.left(), 0.0);
        ctx.surface.put(point.into(), Pixel::new(' ').bg("#F00"));
    }
}

pub fn slider<T: 'static>(
    ctx: &mut Context<'_, T>,
    params: fn(&mut T) -> SliderParams<'_>,
) -> Response<()> {
    Slider::show(params, ctx)
}

struct Background {
    bg: Rgba,
}

impl<T: 'static> View<T> for Background {
    type Args<'a> = Rgba;
    type Response = NoResponse;

    fn create(args: Self::Args<'_>) -> Self {
        Self { bg: args }
    }

    fn update(&mut self, state: &mut T, args: Self::Args<'_>) -> Self::Response {
        self.bg = args;
    }

    fn draw(&mut self, ctx: DrawCtx<T>) {
        ctx.surface.draw(Fill::new(self.bg));
        self.default_draw(ctx);
    }
}

pub fn background<T: 'static, R>(
    bg: impl Into<Rgba>,
    ctx: &mut Context<'_, T>,
    show: impl FnOnce(&mut Context<'_, T>) -> R,
) -> UserResponse<R> {
    Background::show_children(bg.into(), ctx, show)
}

struct Margin {
    margin: geom::Margin,
}

impl<T: 'static> View<T> for Margin {
    type Args<'a> = geom::Margin;
    type Response = NoResponse;

    fn create(args: Self::Args<'_>) -> Self {
        Self { margin: args }
    }

    fn update(&mut self, state: &mut T, args: Self::Args<'_>) -> Self::Response {
        self.margin = args;
    }

    fn layout(&mut self, mut ctx: LayoutCtx<T>, space: Space) -> Size {
        let margin = self.margin.sum();
        let offset = self.margin.left_top().to_point();
        let space = Space {
            min: (space.min - margin).max(Size::ZERO),
            max: (space.max - margin).max(Size::ZERO),
        };

        let mut size = Size::ZERO;
        for &child in ctx.children {
            size = ctx.compute_layout(child, space);
            ctx.set_position(child, offset);
        }
        space.min.max(size.max(margin))
    }
}

pub fn margin<T: 'static, R>(
    margin: impl Into<geom::Margin>,
    ctx: &mut Context<'_, T>,
    show: fn(&mut Context<'_, T>) -> R,
) -> UserResponse<R> {
    Margin::show_children(margin.into(), ctx, show)
}
