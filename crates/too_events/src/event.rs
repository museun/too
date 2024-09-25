use too_math::{Pos2, Vec2};

use crate::{Key, Keybind, Modifiers, MouseButton};

/// An event produced by the user's interaction
#[derive(Clone, Debug, PartialEq)]
#[non_exhaustive]
pub enum Event {
    /// A key was pressed
    KeyPressed {
        key: Key,
        modifiers: Modifiers,
    },
    /// A key was released
    KeyReleased {
        key: Key,
        modifiers: Modifiers,
    },
    /// A key was held down
    KeyRepeat {
        key: Key,
        modifiers: Modifiers,
    },

    /// The mouse cursor has moved
    MouseMove {
        pos: Pos2,
        modifiers: Modifiers,
    },
    /// A mouse button was clicked
    MouseClick {
        pos: Pos2,
        button: MouseButton,
        modifiers: Modifiers,
    },
    /// A mouse button is being held down
    MouseHeld {
        pos: Pos2,
        button: MouseButton,
        modifiers: Modifiers,
    },
    /// A mouse button began dragging
    MouseDragStart {
        pos: Pos2,
        button: MouseButton,
        modifiers: Modifiers,
    },
    /// A mouse button is being dragged
    MouseDragHeld {
        /// The current position of the cursor
        pos: Pos2,
        /// Where the drag started
        origin: Pos2,
        /// The distance the mouse has moved since the last event.
        ///
        /// * `delta.y` is the ___vertical___ direction
        /// * `delta.x` is the ___horizontal___ direction
        ///
        /// * Negative means it was moved down (or left)
        /// * Positive means it was moved up (or right)
        delta: Vec2,
        button: MouseButton,
        modifiers: Modifiers,
    },
    /// A mouse button finished being dragged
    MouseDragRelease {
        /// The current position of the cursor
        pos: Pos2,
        /// Where the drag began
        origin: Pos2,
        button: MouseButton,
        modifiers: Modifiers,
    },
    /// The mouse scroll button was used
    MouseScroll {
        pos: Pos2,
        /// Which direction (and how much) was scolled
        ///
        /// * `delta.y` is the ___vertical___ direction
        /// * `delta.x` is the ___horizontal___ direction
        ///
        /// * Negative means it was scrolled down (or left)
        /// * Positive means it was scrolled up (or right)
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

    /// Was this event a screen switch?
    pub const fn is_screen_switch(&self) -> bool {
        matches!(self, Self::SwitchAltScreen | Self::SwitchMainScreen)
    }

    /// Was this keybind released?
    ///
    /// A [`Keybind`]  can be created from a [`Key`], a `char`, or manually.
    pub fn is_keybind_released(&self, keybind: impl Into<Keybind>) -> bool {
        let Self::KeyReleased { key, modifiers } = *self else {
            return false;
        };
        Self::is_keybind(key, modifiers, keybind)
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

    /// Was this keybind being repeated?
    pub fn is_keybind_repeat(&self, keybind: impl Into<Keybind>) -> bool {
        let Self::KeyRepeat { key, modifiers } = *self else {
            return false;
        };
        Self::is_keybind(key, modifiers, keybind)
    }

    fn is_keybind(key: Key, modifiers: Modifiers, expected: impl Into<Keybind>) -> bool {
        let expected: Keybind = expected.into();
        if matches!(expected.key, Key::Char(..))
            && expected.modifiers.is_none()
            && modifiers.is_none()
        {
            return key == expected.key;
        }
        Keybind::new(key, modifiers) == expected
    }

    /// If this was a mouse event, where was it?
    pub fn mouse_pos(&self) -> Option<Pos2> {
        match self {
            Self::MouseMove { pos, .. }
            | Self::MouseClick { pos, .. }
            | Self::MouseHeld { pos, .. }
            | Self::MouseDragStart { pos, .. }
            | Self::MouseDragHeld { pos, .. }
            | Self::MouseDragRelease { pos, .. }
            | Self::MouseScroll { pos, .. } => Some(*pos),
            _ => None,
        }
    }
}
