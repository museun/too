use crate::{
    math::{Pos2, Vec2},
    view::{Builder, EventCtx, Handled, Interest, Ui, View, ViewEvent},
};

#[derive(Copy, Clone, Debug, Default, PartialEq)]
struct DragState {
    start: Pos2,
    pos: Pos2,
    offset: Vec2,
}

#[derive(Copy, Clone, Debug, Default, PartialEq)]
pub struct DraggingResponse {
    start: Pos2,
    current: Pos2,
    offset: Pos2,
}

impl DraggingResponse {
    pub const fn start(&self) -> Pos2 {
        self.start
    }

    pub const fn current(&self) -> Pos2 {
        self.current
    }

    pub const fn offset(&self) -> Pos2 {
        self.offset
    }
}

#[derive(Copy, Clone, Debug, Default, PartialEq)]
pub struct MouseAreaResponse {
    clicked: bool,
    hovered: bool,
    held: bool,
    pos: Pos2,
    entered: bool,
    leave: bool,
    dragged: Option<DragState>,
    scrolled: Option<Vec2>,
}

impl MouseAreaResponse {
    pub const fn position(&self) -> Pos2 {
        self.pos
    }

    pub const fn clicked(&self) -> bool {
        self.clicked
    }

    pub const fn hovered(&self) -> bool {
        self.hovered
    }

    pub const fn held(&self) -> bool {
        self.held
    }

    pub const fn entered(&self) -> bool {
        self.entered
    }

    pub const fn leave(&self) -> bool {
        self.leave
    }

    pub fn dragged(&self) -> Option<DraggingResponse> {
        self.dragged.as_ref().map(|d| DraggingResponse {
            start: d.start,
            current: d.pos,
            offset: d.pos + d.offset,
        })
    }

    pub const fn scrolled(&self) -> Option<Vec2> {
        self.scrolled
    }
}

#[derive(Copy, Clone, Default, Debug, PartialEq)]
enum MouseState {
    #[default]
    None,
    Hovering,
    Held,
}

#[derive(Default, Debug, Copy, Clone)]
#[must_use = "a view does nothing unless `show()` or `show_children()` is called"]
pub struct MouseArea {
    state: MouseState,
    clicked: bool,
    pos: Pos2,
    entered: bool,
    leave: bool,
    scrolled: Option<Vec2>,
    dragged: Option<DragState>,
}

impl MouseArea {
    pub fn new() -> Self {
        Self::default()
    }
}

impl<'v> Builder<'v> for MouseArea {
    type View = Self;
}

impl View for MouseArea {
    type Args<'v> = Self;
    type Response = MouseAreaResponse;

    fn create(builder: Self::Args<'_>) -> Self {
        builder
    }

    fn update(&mut self, _: Self::Args<'_>, _: &Ui) -> Self::Response {
        let state = std::mem::take(&mut self.state);
        let dragged = if !matches!(state, MouseState::Held) {
            self.dragged.take()
        } else {
            self.dragged
        };

        MouseAreaResponse {
            pos: self.pos,
            clicked: std::mem::take(&mut self.clicked),
            hovered: matches!(state, MouseState::Hovering),
            held: matches!(state, MouseState::Held),
            dragged,
            scrolled: std::mem::take(&mut self.scrolled),
            entered: self.entered,
            leave: self.leave,
        }
    }

    fn interactive(&self) -> bool {
        true
    }

    fn interests(&self) -> Interest {
        Interest::MOUSE_INSIDE
    }

    fn event(&mut self, event: ViewEvent, ctx: EventCtx) -> Handled {
        match event {
            ViewEvent::MouseMove { pos, .. } => {
                self.state = MouseState::None;
                self.pos = pos;
                self.dragged.take();
            }

            ViewEvent::MouseDrag { start, current, .. } => {
                self.state = MouseState::Held;
                self.pos = current;

                let rect = ctx.rect();
                self.dragged
                    .get_or_insert_with(|| DragState {
                        start,
                        pos: current,
                        offset: (rect.min - current).to_vec2(),
                    })
                    .pos = current;
            }

            ViewEvent::MouseClicked { pos, .. } => {
                self.state = MouseState::Held;
                self.pos = pos;
                self.clicked = true
            }

            // ViewEvent::MouseButtonChanged {
            //     pos,
            //     button,
            //     inside,
            //     down,
            //     ..
            // } if inside && (down || matches!(self.state, MouseState::Held)) => {
            //     self.clicked = true;
            //     self.state = MouseState::Held;
            //     if !down {
            //         self.dragged.take();
            //     }
            // }
            ViewEvent::MouseScroll { delta, .. } => {
                self.scrolled = Some(delta);
                self.dragged.take();
            }

            ViewEvent::MouseEntered => {
                self.entered = true;
                self.state = MouseState::Hovering;
                self.dragged.take();
            }

            ViewEvent::MouseLeave => {
                self.leave = true;
                self.state = MouseState::None;
                self.dragged.take();
            }
            _ => return Handled::Bubble,
        }

        Handled::Bubble
    }
}

pub fn mouse_area() -> MouseArea {
    MouseArea::default()
}
