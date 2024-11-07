use crate::{math::Vec2, Renderer};

mod event;
pub use event::Event;

mod key;
pub use key::Key;

mod keybind;
pub use keybind::Keybind;

mod modifiers;
pub use modifiers::Modifiers;

mod mouse_button;
pub use mouse_button::MouseButton;

mod current_screen;
pub use current_screen::CurrentScreen;

mod command;
pub use command::Command;

/// An abstraction over a writable backend
pub trait Backend {
    /// The writer for this backend
    type Renderer<'a>: Renderer
    where
        Self: 'a;
    /// The current size of the backends 'screen'
    fn size(&self) -> Vec2;
    /// Can we draw to this backend, currently?
    fn should_draw(&self) -> bool;
    /// Send a [`Command`] to this backend
    fn command(&mut self, cmd: Command);
    /// Get the writer for this backend
    fn writer(&mut self) -> Self::Renderer<'_>;
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
