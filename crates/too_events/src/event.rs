use too_math::{Pos2, Vec2};

use crate::{Key, Keybind, Modifiers, MouseButton};

#[derive(Clone, Debug, PartialEq)]
#[non_exhaustive]
pub enum Event {
    KeyPressed {
        key: Key,
        modifiers: Modifiers,
    },
    KeyReleased {
        key: Key,
        modifiers: Modifiers,
    },
    KeyRepeat {
        key: Key,
        modifiers: Modifiers,
    },

    MouseMove {
        pos: Pos2,
        modifiers: Modifiers,
    },
    MouseClick {
        pos: Pos2,
        button: MouseButton,
        modifiers: Modifiers,
    },
    MouseHeld {
        pos: Pos2,
        button: MouseButton,
        modifiers: Modifiers,
    },
    MouseDragStart {
        pos: Pos2,
        button: MouseButton,
        modifiers: Modifiers,
    },
    MouseDragHeld {
        pos: Pos2,
        origin: Pos2,
        delta: Vec2,
        button: MouseButton,
        modifiers: Modifiers,
    },
    MouseDragRelease {
        pos: Pos2,
        origin: Pos2,
        button: MouseButton,
        modifiers: Modifiers,
    },
    MouseScroll {
        pos: Pos2,
        delta: Vec2,
        modifiers: Modifiers,
    },

    Resize(Vec2),
    FocusGained,
    FocusLost,
    Paste(String),
    SwitchAltScreen,
    SwitchMainScreen,
    Quit,
}

impl Event {
    pub const fn is_quit(&self) -> bool {
        matches!(self, Self::Quit)
    }

    pub const fn is_screen_switch(&self) -> bool {
        matches!(self, Self::SwitchAltScreen | Self::SwitchMainScreen)
    }

    pub fn is_keybind_up(&self, keybind: impl Into<Keybind>) -> bool {
        let Self::KeyReleased { key, modifiers } = *self else {
            return false;
        };
        Self::is_keybind(key, modifiers, keybind)
    }

    pub fn is_keybind_pressed(&self, keybind: impl Into<Keybind>) -> bool {
        let Self::KeyPressed { key, modifiers } = *self else {
            return false;
        };
        Self::is_keybind(key, modifiers, keybind)
    }

    pub fn is_keybind_repeat(&self, keybind: impl Into<Keybind>) -> bool {
        let Self::KeyRepeat { key, modifiers } = *self else {
            return false;
        };
        Self::is_keybind(key, modifiers, keybind)
    }

    fn is_keybind(key: Key, modifiers: Modifiers, expected: impl Into<Keybind>) -> bool {
        let have = Keybind::new(key, modifiers);
        have == expected.into()
    }
}
