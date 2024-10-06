#![cfg_attr(debug_assertions, allow(dead_code, unused_variables,))] // <-- the only one you need
use std::collections::VecDeque;

mod backend;
pub use backend::{
    Backend, Command, CurrentScreen, DummyBackend, Event, EventReader, Key, Keybind, Modifiers,
    MouseButton,
};

pub mod ema_window;

pub mod math;

pub mod overlay;

mod renderer;
pub use renderer::{
    rgba, Attribute, Cell, Color, DebugRenderer, Gradient, Grapheme, Pixel, Renderer, Rgba,
    Surface, TermRenderer, Underline,
};

pub mod layout;

mod text;
pub use text::{Justification, MeasureText, Text};

pub mod animation;
pub use animation::AnimationManager;

pub mod index;
#[doc(inline)]
pub use index::Index;

pub mod view;

#[cfg(feature = "terminal")]
pub mod term;

pub mod hasher;

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
