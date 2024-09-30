use crate::{
    math::{vec2, Vec2},
    Backend, Command, Event, EventReader,
};

/// A dummy backend that does nothing
pub struct DummyBackend;

impl Backend for DummyBackend {
    type Out<'a> = std::io::Empty;

    fn size(&self) -> Vec2 {
        vec2(80, 25)
    }

    fn should_draw(&self) -> bool {
        true
    }

    fn command(&mut self, _cmd: Command) {}

    fn writer(&mut self) -> Self::Out<'_> {
        std::io::empty()
    }
}

impl EventReader for DummyBackend {
    fn try_read_event(&mut self) -> Option<Event> {
        Some(Event::Quit)
    }
}
