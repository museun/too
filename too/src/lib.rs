mod backend;
use std::collections::VecDeque;

pub use backend::{
    Backend, Command, CurrentScreen, DummyBackend, Event, EventReader, Key, Keybind, Modifiers,
    MouseButton, MouseState, TemporalMouseEvent,
};

pub mod ema_window;

pub mod math;

pub mod overlay;

mod renderer;
pub use renderer::{
    rgba, Attribute, Canvas, Cell, Color, CroppedSurface, DebugRenderer, Gradient, Grapheme, Pixel,
    Renderer, Rgba, Surface, TermRenderer,
};

pub mod layout;

mod text;
pub use text::{Justification, MeasureText, Text};

pub mod animation;

pub mod index;
#[doc(inline)]
pub use index::Index;

pub mod view;

#[derive(Default)]
pub struct Commands {
    inner: VecDeque<Command>,
}

impl Commands {
    pub fn push(&mut self, cmd: Command) {
        self.inner.push_back(cmd);
    }

    pub fn drain(&mut self) -> impl ExactSizeIterator<Item = Command> + '_ {
        self.inner.drain(..)
    }
}
