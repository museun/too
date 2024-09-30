// TODO don't glob import everything

mod backend;
pub use backend::{
    Backend, Command, CurrentScreen, DummyBackend, Event, EventReader, Key, Keybind, Modifiers,
    MouseButton, MouseState, TemporalMouseEvent,
};

// TODO this should be placed elsewhere
// TODO also change the name of `App` and `AppRunner`
mod immediate;
pub use immediate::*;

pub mod ema_window;

pub mod math;

pub mod overlay;

// TODO this should also sort of be its own module
mod renderer;
pub use renderer::*;

mod runner;
pub use runner::{Context, Runner};

pub mod layout;
