#![cfg_attr(debug_assertions, allow(dead_code, unused_variables,))]
use std::{collections::VecDeque, ops::Deref};

mod backend;
pub use backend::{
    Backend, Command, CurrentScreen, DummyBackend, Event, EventReader, Key, Keybind, Modifiers,
    MouseButton,
};

pub mod math;

mod renderer;
use compact_str::{CompactString, ToCompactString};
pub use renderer::{
    rgba, Attribute, Border, Cell, Color, DebugRenderer, DummyRenderer, Gradient, Grapheme, Pixel,
    Renderer, Rgba, Surface, TermRenderer, Underline,
};

pub mod layout;

mod text;
pub use text::{MeasureText, Text};

pub mod animation;
pub use animation::Animations;

pub mod view;
pub mod views;

mod rasterizer;

// TODO get rid of this
use crate::math::Size;
#[inline(always)]
#[deprecated(note = "don't use this, use Text when its implemented")]
pub fn measure_text(data: &str) -> Size {
    use unicode_width::UnicodeWidthStr as _;
    Size::new(data.width() as f32, 1.0)
}

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

#[doc(hidden)]
pub use compact_str::format_compact as __dont_use_this_because_semver;
// basically just https://github.com/ParkMyCar/compact_str/blob/193d13eaa5a92b3c39c2f7289dc44c95f37c80d1/compact_str/src/macros.rs#L28
// but semver-safe
/// Like [`std::format!`] but for a [`Str`]
#[macro_export]
macro_rules! format_str {
    ($($arg:tt)*) => {
        $crate::Str::from($crate::__dont_use_this_because_semver!($($arg)*))
    }
}

/// A semver wrapper around a [`CompactString`](https://docs.rs/compact_str/0.8.0/compact_str/index.html)
///
/// You would normally not need to name this type, anything that implements [`ToCompactString`](https://docs.rs/compact_str/0.8.0/compact_str/trait.ToCompactString.html) can be turned into this type.
///
/// You can use [`format_str!`] like [`std::format!`] to make this type
/// - or `Str::from(&str)`
/// - or `Str::from(String)`
/// - or `Str::from(usize)`
/// - etc
#[derive(Clone, Debug, PartialEq, PartialOrd, Eq, Ord, Hash)]
pub struct Str(CompactString);

impl Str {
    pub const fn new(str: &'static str) -> Self {
        Self(CompactString::const_new(str))
    }
}

impl AsRef<str> for Str {
    fn as_ref(&self) -> &str {
        self.0.as_str()
    }
}

impl Deref for Str {
    type Target = str;
    fn deref(&self) -> &Self::Target {
        self.0.as_str()
    }
}

impl<T> From<T> for Str
where
    T: ToCompactString,
{
    fn from(value: T) -> Self {
        Self(value.to_compact_string())
    }
}

fn foo() {
    let a = 1;
    let t = format_str!("hello {a}");
}
