use crate::view::{view::Context, Event, EventCtx, Handled, Interest, UpdateCtx, View, ViewExt};

struct EventArea {
    last: Option<Event>,
}

impl<T: 'static> View<T> for EventArea {
    type Args<'a> = ();
    type Response = Option<Event>;

    fn create(args: Self::Args<'_>) -> Self {
        Self { last: None }
    }

    fn update(&mut self, ctx: UpdateCtx<T>, args: Self::Args<'_>) -> Self::Response {
        self.last.take()
    }

    fn interest(&self) -> Interest {
        Interest::MOUSE | Interest::KEY_INPUT
    }

    fn event(&mut self, ctx: EventCtx<T>, event: &Event) -> Handled {
        match event {
            Event::MouseEnter(..)
            | Event::MouseLeave(..)
            | Event::MouseMove(..)
            | Event::MouseClick(..)
            | Event::MouseHeld(..)
            | Event::MouseDragStart(..)
            | Event::MouseDragHeld(..)
            | Event::MouseDragRelease(..)
            | Event::MouseScroll(..)
            | Event::KeyInput(..) => self.last = Some(*event),
            _ => {}
        }
        Handled::Bubble
    }
}

pub fn event_area<T: 'static, R>(
    ctx: &mut Context<T>,
    show: impl FnOnce(&mut Context<T>) -> R,
) -> Option<Event> {
    let (resp, _) = EventArea::show_children((), ctx, show);
    resp
}
