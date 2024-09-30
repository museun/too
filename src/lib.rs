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
    shapes, Attribute, DebugRenderer, Gradient, Pixel, PixelColor, Renderer, Rgba, Shape, Surface,
    SurfaceMut, TermRenderer,
};

mod runner;
pub use runner::{Context, Runner};

pub mod layout;
