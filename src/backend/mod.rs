use crate::math::Vec2;

mod events;
pub use events::{
    event::Event,
    key::Key,
    key_bind::Keybind,
    modifiers::Modifiers,
    mouse_button::MouseButton,
    mouse_event::{MouseState, TemporalMouseEvent},
};

mod current_screen;
pub use current_screen::CurrentScreen;

mod command;
pub use command::Command;

/// An abstraction over a writable backend
pub trait Backend {
    /// The writer for this backend
    type Out<'a>: std::io::Write
    where
        Self: 'a;
    /// The current size of the backends 'screen'
    fn size(&self) -> Vec2;
    /// Can we draw to this backend, currently?
    fn should_draw(&self) -> bool;
    /// Send a [`Command`] to this backend
    fn command(&mut self, cmd: Command);
    /// Get the writer for this backend
    fn writer(&mut self) -> Self::Out<'_>;
}

/// A trait to read [`Event`]s
pub trait EventReader {
    /// Tries to read an [`Event`]
    ///
    /// This'll return None if an event isn't ready
    fn try_read_event(&mut self) -> Option<Event>;
}

mod dummy;
pub use dummy::DummyBackend;
