mod backend;
pub use backend::{
    Backend, Command, CurrentScreen, DummyBackend, Event, EventReader, Key, Keybind, Modifiers,
    MouseButton, MouseState, TemporalMouseEvent,
};

mod immediate;
pub use immediate::{App, AppRunner};

pub mod ema_window;

pub mod math;

pub mod overlay;

mod renderer;
pub use renderer::{
    rgba, Attribute, Cell, Color, DebugRenderer, Gradient, Grapheme, Pixel, Renderer, Rgba,
    Surface, TermRenderer,
};

mod runner;
pub use runner::{Context, Runner};

pub mod layout;

mod text;
pub use text::{Justification, Text};

pub mod animation;

pub mod index;
#[doc(inline)]
pub use index::Index;
