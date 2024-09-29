use crate::{
    geom::{Point, Vector},
    view::Context,
    Event, EventCtx, Handled, Interest, Response, UpdateCtx, View, ViewExt,
};

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

    fn update(&mut self, ctx: UpdateCtx<T>, args: Self::Args<'_>) -> Self::Response {
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

// TODO hscroll
pub fn on_scroll<T: 'static, R>(
    ctx: &mut Context<T>,
    show: impl FnOnce(&mut Context<T>) -> R,
) -> Response<Option<f32>, R> {
    let filter = const { MouseEvent::empty().scroll() };
    let resp = mouse_area(filter, ctx, show);
    resp.map(|resp, inner| (resp.scrolled, inner))
}
