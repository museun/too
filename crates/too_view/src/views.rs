use std::ops::RangeInclusive;

use too_backend::{Key, Keybind, Modifiers};
use too_math::{layout::Align2, vec2};
use too_renderer::{
    shapes::{Fill, Text},
    Pixel, Rgba,
};

use crate::{
    geom::{self, Point, Size, Space, Vector},
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
        use too_renderer::shapes::Label as _;
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
    L: too_renderer::shapes::Label + 'static + Clone,
{
    label: L,
}

impl<L, T> View<T> for StaticLabel<L>
where
    L: too_renderer::shapes::Label + 'static + Clone,
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
    label: impl too_renderer::shapes::Label + 'static + Clone,
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
        // TODO this is all wrong
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
    show: impl FnOnce(&mut Context<'_, T>) -> R,
) -> UserResponse<R> {
    Margin::show_children(margin.into(), ctx, show)
}

pub struct ProgressBarParams<'a> {
    pub value: &'a mut f32,
    pub range: RangeInclusive<f32>,
}

impl<'a> ProgressBarParams<'a> {
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

struct Progress<T: 'static> {
    params: fn(&mut T) -> ProgressBarParams<'_>,
}

impl<T: 'static> Progress<T> {
    fn normalize(value: f32, range: &RangeInclusive<f32>) -> f32 {
        let value = value.clamp(*range.start(), *range.end());
        (value - range.start()) / (range.end() - range.start())
    }

    fn denormalize(value: f32, range: &RangeInclusive<f32>) -> f32 {
        let value = value.clamp(0.0, 1.0);
        value * (range.end() - range.start()) + range.start()
    }
}

impl<T: 'static> View<T> for Progress<T> {
    type Args<'a> = fn(&mut T) -> ProgressBarParams<'_>;
    type Response = NoResponse;

    fn create(args: Self::Args<'_>) -> Self {
        Self { params: args }
    }

    fn update(&mut self, state: &mut T, args: Self::Args<'_>) -> Self::Response {
        self.params = args
    }

    fn layout(&mut self, ctx: LayoutCtx<T>, space: Space) -> Size {
        // TODO axis
        space.fit(Size::new(20.0, 1.0))
    }

    fn draw(&mut self, ctx: DrawCtx<T>) {
        let params = (self.params)(ctx.state);

        let (min, max) = (ctx.rect.left(), ctx.rect.right());
        let x = Self::normalize(*params.value, &params.range);
        let x = min + (x * (max - min));

        ctx.surface.draw(Fill::new("#555"));

        // surface::crop does not work -- we need to normalize our rect to 0,0
        for x in 0..(x - ctx.rect.left()).round() as i32 {
            ctx.surface
                .put(too_math::pos2(x, 0), Pixel::new(' ').bg("#FFF"));
        }
    }
}

pub fn progress_bar<T: 'static>(
    ctx: &mut Context<'_, T>,
    params: fn(&mut T) -> ProgressBarParams<'_>,
) -> Response<()> {
    Progress::show(params, ctx)
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct ButtonResponse {
    pub clicked: bool,
}

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
    type Response = ButtonResponse;

    fn create(args: Self::Args<'_>) -> Self {
        Self {
            params: args,
            state: ButtonState::None,
        }
    }

    fn update(&mut self, state: &mut T, args: Self::Args<'_>) -> Self::Response {
        let clicked = match self.state {
            ButtonState::Clicked => {
                self.state = ButtonState::Hovered;
                true
            }
            _ => false,
        };

        self.params = args;
        ButtonResponse { clicked }
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
        space.fit(Size::from(size) + Vector::new(2.0, 0.0))
    }

    fn draw(&mut self, ctx: DrawCtx<T>) {
        let params = (self.params)(ctx.state);

        let fg = if params.enabled { "#FFF" } else { "#AAA" };
        let bg = match self.state {
            ButtonState::Hovered if params.enabled => "#F00",
            ButtonState::Held if params.enabled => "#F0F",
            ButtonState::Clicked if params.enabled => "#00F",
            ButtonState::None if params.enabled => "#333",
            _ => "#333",
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
) -> Response<ButtonResponse> {
    Button::show(params, ctx)
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct CheckboxResponse {
    pub selected: bool,
}

pub struct CheckboxParams<'a> {
    pub label: &'a str,
    pub value: &'a mut bool,
}

impl<'a> CheckboxParams<'a> {
    pub fn new(label: &'a str, value: &'a mut bool) -> Self {
        Self { label, value }
    }
}

struct Checkbox<T: 'static> {
    params: fn(&mut T) -> CheckboxParams<'_>,
    state: ButtonState,
}

impl<T: 'static> View<T> for Checkbox<T> {
    type Args<'a> = fn(&mut T) -> CheckboxParams<'_>;
    type Response = CheckboxResponse;

    fn create(args: Self::Args<'_>) -> Self {
        Self {
            params: args,
            state: ButtonState::None,
        }
    }

    fn update(&mut self, state: &mut T, args: Self::Args<'_>) -> Self::Response {
        self.params = args;

        let clicked = match self.state {
            ButtonState::Clicked => {
                self.state = ButtonState::Hovered;
                true
            }
            _ => false,
        };

        CheckboxResponse {
            selected: *(self.params)(state).value,
        }
    }

    fn interest(&self) -> Interest {
        Interest::MOUSE
    }

    fn event(&mut self, ctx: EventCtx<T>, event: &Event) -> Handled {
        self.state = match event {
            Event::MouseEnter(..) => ButtonState::Hovered,
            Event::MouseLeave(..) => ButtonState::None,
            Event::MouseClick(..) => ButtonState::Clicked,
            Event::MouseHeld(..) => ButtonState::Held,
            _ => return Handled::Bubble,
        };

        Handled::Sink
    }

    fn layout(&mut self, ctx: LayoutCtx<T>, space: Space) -> Size {
        let params = (self.params)(ctx.state);
        let size = Text::new(params.label).size();
        space.fit(Size::from(size) + Vector::new(3.0, 0.0))
    }

    fn draw(&mut self, ctx: DrawCtx<T>) {
        let params = (self.params)(ctx.state);

        let bg = match self.state {
            ButtonState::Hovered => "#F00",
            ButtonState::Held => "#F0F",
            ButtonState::Clicked => "#00F",
            ButtonState::None => "#333",
        };

        // ☒
        // ☐

        let offset = ctx.surface.rect().translate(vec2(3, 0));
        ctx.surface
            .draw(Fill::new(bg))
            .crop(offset) // this is such a hack
            .draw(Text::new(params.label).fg("#FFF"));
    }
}

pub fn checkbox<T: 'static>(
    ctx: &mut Context<'_, T>,
    params: fn(&mut T) -> CheckboxParams<'_>,
) -> Response<CheckboxResponse> {
    Checkbox::show(params, ctx)
}

#[derive(Copy, Clone, Default, Debug, PartialEq)]
pub struct MouseEvent(u8);
impl MouseEvent {
    pub const EMPTY: Self = Self(0);
    pub const ALL: Self = Self(
        Self::ENTER.0
            | Self::LEAVE.0
            | Self::MOVE.0
            | Self::DRAG.0
            | Self::CLICK.0
            | Self::HELD.0
            | Self::SCROLL.0,
    );

    pub const ENTER: Self = Self(1 << 0);
    pub const LEAVE: Self = Self(1 << 1);
    pub const MOVE: Self = Self(1 << 2);
    pub const DRAG: Self = Self(1 << 3);
    pub const CLICK: Self = Self(1 << 4);
    pub const HELD: Self = Self(1 << 5);
    pub const SCROLL: Self = Self(1 << 6);
}

impl MouseEvent {
    pub const fn empty() -> Self {
        Self::EMPTY
    }

    pub const fn enter(self) -> Self {
        Self(self.0 | Self::ENTER.0)
    }
    pub const fn leave(self) -> Self {
        Self(self.0 | Self::LEAVE.0)
    }
    pub const fn moved(self) -> Self {
        Self(self.0 | Self::MOVE.0)
    }
    pub const fn drag(self) -> Self {
        Self(self.0 | Self::DRAG.0)
    }
    pub const fn click(self) -> Self {
        Self(self.0 | Self::CLICK.0)
    }
    pub const fn held(self) -> Self {
        Self(self.0 | Self::HELD.0)
    }
    pub const fn scroll(self) -> Self {
        Self(self.0 | Self::SCROLL.0)
    }
}

impl MouseEvent {
    pub const fn is_enter(&self) -> bool {
        (self.0 & Self::ENTER.0) != 0
    }
    pub const fn is_leave(&self) -> bool {
        (self.0 & Self::LEAVE.0) != 0
    }
    pub const fn is_move(&self) -> bool {
        (self.0 & Self::MOVE.0) != 0
    }
    pub const fn is_drag(&self) -> bool {
        (self.0 & Self::DRAG.0) != 0
    }
    pub const fn is_click(&self) -> bool {
        (self.0 & Self::CLICK.0) != 0
    }
    pub const fn is_held(&self) -> bool {
        (self.0 & Self::HELD.0) != 0
    }
    pub const fn is_scroll(&self) -> bool {
        (self.0 & Self::SCROLL.0) != 0
    }
}
impl std::ops::BitOr for MouseEvent {
    type Output = Self;
    fn bitor(self, rhs: Self) -> Self::Output {
        Self(self.0 | rhs.0)
    }
}
impl std::ops::BitAnd for MouseEvent {
    type Output = Self;
    fn bitand(self, rhs: Self) -> Self::Output {
        Self(self.0 & rhs.0)
    }
}
impl std::ops::BitXor for MouseEvent {
    type Output = Self;
    fn bitxor(self, rhs: Self) -> Self::Output {
        Self(self.0 ^ rhs.0)
    }
}

impl std::ops::BitOrAssign for MouseEvent {
    fn bitor_assign(&mut self, rhs: Self) {
        *self = *self | rhs
    }
}
impl std::ops::BitAndAssign for MouseEvent {
    fn bitand_assign(&mut self, rhs: Self) {
        *self = *self & rhs
    }
}
impl std::ops::BitXorAssign for MouseEvent {
    fn bitxor_assign(&mut self, rhs: Self) {
        *self = *self ^ rhs
    }
}

impl std::ops::Not for MouseEvent {
    type Output = Self;
    fn not(self) -> Self::Output {
        Self(!self.0)
    }
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Dragged {
    pub origin: Point,
    pub current: Point,
    pub delta: Vector,
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct MouseAreaResponse {
    pub clicked: bool,
    pub hovered: bool,
    pub held: bool,
    pub scrolled: Option<f32>,
    pub dragged: Option<Dragged>,
}

#[derive(Default)]
enum MouseState {
    #[default]
    None,
    Hovering,
    Held,
}

#[derive(Default)]
struct MouseArea {
    filter: MouseEvent,
    state: MouseState,
    clicked: bool,
    scrolled: Option<f32>,
    dragged: Option<Dragged>,
}

impl MouseArea {
    fn reset(&mut self) {
        std::mem::take(&mut self.state);
        std::mem::take(&mut self.clicked);
        std::mem::take(&mut self.scrolled);
        std::mem::take(&mut self.dragged);
    }
}

impl<T: 'static> View<T> for MouseArea {
    type Args<'a> = MouseEvent;
    type Response = MouseAreaResponse;

    fn create(args: Self::Args<'_>) -> Self {
        Self {
            filter: args,
            ..Self::default()
        }
    }

    fn update(&mut self, state: &mut T, args: Self::Args<'_>) -> Self::Response {
        self.filter = args;
        let resp = MouseAreaResponse {
            clicked: std::mem::take(&mut self.clicked),
            hovered: matches!(self.state, MouseState::Hovering),
            held: matches!(self.state, MouseState::Held),
            scrolled: self.scrolled,
            dragged: self.dragged,
        };
        self.reset();
        resp
    }

    fn interest(&self) -> Interest {
        Interest::MOUSE
    }

    fn event(&mut self, ctx: EventCtx<T>, event: &Event) -> Handled {
        self.reset();

        // TODO support different buttons
        match event {
            Event::MouseEnter(ev) if self.filter.is_enter() => self.state = MouseState::Hovering,
            Event::MouseLeave(ev) if self.filter.is_leave() => {}
            Event::MouseClick(ev) if self.filter.is_click() && ev.button.is_primary() => {
                self.clicked = true;
                self.state = MouseState::Hovering
            }
            Event::MouseHeld(ev) if self.filter.is_held() && ev.button.is_primary() => {
                self.state = MouseState::Held
            }
            Event::MouseDrag(ev) if self.filter.is_drag() && ev.button.is_primary() => {
                self.dragged = Some(Dragged {
                    origin: ev.origin,
                    current: ev.pos,
                    delta: ev.delta,
                })
            }

            // TODO hscroll
            Event::MouseScroll(ev) if self.filter.is_scroll() => self.scrolled = Some(ev.delta.y),
            _ => {}
        };

        Handled::Bubble
    }
}

pub fn mouse_area<T: 'static, R>(
    filter: MouseEvent,
    ctx: &mut Context<T>,
    show: impl FnOnce(&mut Context<T>) -> R,
) -> Response<MouseAreaResponse, R> {
    MouseArea::show_children(filter, ctx, show)
}

pub fn on_click<T: 'static, R>(
    ctx: &mut Context<T>,
    show: impl FnOnce(&mut Context<T>) -> R,
) -> Response<bool, R> {
    let filter = const { MouseEvent::empty().click() };
    let resp = mouse_area(filter, ctx, show);
    resp.map(|resp, inner| (resp.clicked, inner))
}

pub fn on_drag<T: 'static, R>(
    ctx: &mut Context<T>,
    show: impl FnOnce(&mut Context<T>) -> R,
) -> Response<Option<Dragged>, R> {
    let filter = const { MouseEvent::empty().drag() };
    let resp = mouse_area(filter, ctx, show);
    resp.map(|resp, inner| (resp.dragged, inner))
}

// TODO h-scroll
pub fn on_scroll<T: 'static, R>(
    ctx: &mut Context<T>,
    show: impl FnOnce(&mut Context<T>) -> R,
) -> Response<Option<f32>, R> {
    let filter = const { MouseEvent::empty().scroll() };
    let resp = mouse_area(filter, ctx, show);
    resp.map(|resp, inner| (resp.scrolled, inner))
}

// row
// column
// float
// flex
// constrained
// unconstrained
// text input
// border
// radio
// checkbox (wip)
// todo value
