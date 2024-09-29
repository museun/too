use std::fmt::Write;

use crate::{Key, Modifiers};

/// A keybind is a combination of a [`Key`] and some [`Modifiers`]
#[derive(Copy, Clone, PartialEq, Eq, Hash)]
pub struct Keybind {
    pub key: Key,
    pub modifiers: Modifiers,
}

impl std::fmt::Debug for Keybind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Keybind")
            .field("key", &self.key)
            .field("modifiers", &self.modifiers)
            .finish()
    }
}

impl Keybind {
    pub const fn new(key: Key, modifiers: Modifiers) -> Self {
        Self { key, modifiers }
    }

    pub const fn from_key(key: Key) -> Self {
        Self::new(key, Modifiers::NONE)
    }

    pub const fn from_char(char: char) -> Self {
        Self::from_key(Key::Char(char))
    }

    pub const fn ctrl(mut self) -> Self {
        self.modifiers = Modifiers(self.modifiers.0 | Modifiers::CTRL.0);
        self
    }

    pub const fn shift(mut self) -> Self {
        self.modifiers = Modifiers(self.modifiers.0 | Modifiers::SHIFT.0);
        self
    }

    pub const fn alt(mut self) -> Self {
        self.modifiers = Modifiers(self.modifiers.0 | Modifiers::ALT.0);
        self
    }
}

impl From<char> for Keybind {
    fn from(value: char) -> Self {
        let mut this = Self::from_char(value);
        if value.is_ascii_uppercase() {
            this = this.shift();
        }
        this
    }
}

impl From<Key> for Keybind {
    fn from(value: Key) -> Self {
        Self::from_key(value)
    }
}

impl std::fmt::Display for Keybind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        const FIELDS: [&str; 3] = ["Shift", "Ctrl", "Alt"];

        let mut seen = false;
        for (flag, repr) in (0..).zip(FIELDS) {
            if (self.modifiers.0 >> flag) & 1 == 1 {
                if seen {
                    f.write_str(" + ")?;
                }
                f.write_str(repr)?;
                seen |= true
            }
        }

        if seen {
            f.write_str(" + ")?;
        }

        match self.key {
            Key::Char(' ') => f.write_str("Space"),
            Key::Char(c) => f.write_char(c),
            Key::Function(n) => f.write_fmt(format_args!("F{n}")),
            Key::Left => f.write_str("Left"),
            Key::Right => f.write_str("Right"),
            Key::Up => f.write_str("Up"),
            Key::Down => f.write_str("Down"),
            Key::PageUp => f.write_str("PageUp"),
            Key::PageDown => f.write_str("PageDown"),
            Key::Home => f.write_str("Home"),
            Key::End => f.write_str("End"),
            Key::Insert => f.write_str("Insert"),
            Key::Enter => f.write_str("Enter"),
            Key::Delete => f.write_str("Delete"),
            Key::Backspace => f.write_str("Backspace"),
            Key::Escape => f.write_str("Escape"),
            Key::Tab => f.write_str("Tab"),
        }
    }
}
