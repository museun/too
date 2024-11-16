use crate::{
    math::{Pos2, Vec2},
    Key, Keybind, Modifiers, MouseButton,
};

/// An event produced by the user's interaction
#[derive(Clone, Debug, PartialEq)]
#[non_exhaustive]
pub enum Event {
    /// A key was pressed
    KeyPressed {
        key: Key,
        modifiers: Modifiers,
    },

    MouseButtonChanged {
        pos: Pos2,
        button: MouseButton,
        down: bool,
        modifiers: Modifiers,
    },

    MouseDrag {
        pos: Pos2,
        button: MouseButton,
        modifiers: Modifiers,
    },

    MouseMove {
        pos: Pos2,
    },

    MouseScroll {
        delta: Vec2,
        modifiers: Modifiers,
    },

    /// The screen has resized
    Resize(Vec2),

    FocusGained,
    FocusLost,
    Paste(String),

    /// The screen was switched to the alt screen (the one that is used for drawing)
    SwitchAltScreen,
    /// The screen was switched to the main screen (the one not used for drawing)
    SwitchMainScreen,
    /// The backend has exited, you should clean up
    Quit,
}

impl Event {
    /// Was this event a quit?
    pub const fn is_quit(&self) -> bool {
        matches!(self, Self::Quit)
    }

    pub const fn is_key_event(&self) -> bool {
        matches!(self, Self::KeyPressed { .. })
    }

    pub const fn is_mouse_event(&self) -> bool {
        matches!(
            self,
            Self::MouseMove { .. }
                | Self::MouseButtonChanged { .. }
                | Self::MouseScroll { .. }
                | Self::MouseDrag { .. }
        )
    }

    /// Was this event a screen switch?
    pub const fn is_screen_switch(&self) -> bool {
        matches!(self, Self::SwitchAltScreen | Self::SwitchMainScreen)
    }

    /// Was this keybind pressed?
    ///
    /// A [`Keybind`]  can be created from a [`Key`], a `char`, or manually.
    pub fn is_keybind_pressed(&self, keybind: impl Into<Keybind>) -> bool {
        let Self::KeyPressed { key, modifiers } = *self else {
            return false;
        };
        Self::is_keybind(key, modifiers, keybind)
    }

    // TODO redo this
    // if we remove 'shift' from modifiers (means splitting it into Key and Mouse modifiers)
    // we can normalize the uppercase when we construct the `Key`
    fn is_keybind(key: Key, modifiers: Modifiers, expected: impl Into<Keybind>) -> bool {
        let mut expected: Keybind = expected.into();

        if let Key::Char(ch) = expected.key {
            if ch.is_alphanumeric() {
                let mut keybind = Keybind::new(key, modifiers);

                if keybind.key.is_ascii_lowercase() && keybind.modifiers.is_shift() {
                    if let Some(key) = keybind.key.to_ascii_uppercase() {
                        keybind.key = key;
                    }
                }

                if expected.key.is_ascii_lowercase() && expected.modifiers.is_shift() {
                    if let Some(key) = expected.key.to_ascii_uppercase() {
                        expected.key = key;
                    }
                }

                keybind.modifiers = modifiers.remove(Modifiers::SHIFT);
                expected.modifiers = expected.modifiers.remove(Modifiers::SHIFT);

                return keybind == expected;
            }
        }

        if matches!(expected.key, Key::Char(..))
            && (expected.modifiers.is_none() || expected.modifiers.is_shift_only())
            && (modifiers.is_none() || modifiers.is_shift_only())
        {
            return key == expected.key;
        }

        Keybind::new(key, modifiers) == expected
    }

    /// Were any keyboard modifiers involved in this event?
    pub const fn modifiers(&self) -> Option<Modifiers> {
        match self {
            Self::KeyPressed { modifiers, .. }
            | Self::MouseButtonChanged { modifiers, .. }
            | Self::MouseDrag { modifiers, .. }
            | Self::MouseScroll { modifiers, .. } => Some(*modifiers),
            _ => None,
        }
    }
}
