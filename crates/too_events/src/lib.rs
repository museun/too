pub trait EventReader {
    fn try_read_event(&mut self) -> Option<Event>;
}

mod event;
pub use event::Event;

mod key;
pub use key::Key;

mod modifiers;
pub use modifiers::Modifiers;

mod mouse_button;
pub use mouse_button::MouseButton;

mod mouse_event;
pub use mouse_event::{MouseState, TemporalEvent};

mod key_bind;
pub use key_bind::Keybind;
