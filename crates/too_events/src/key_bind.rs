use crate::{Key, Modifiers};

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub struct Keybind {
    pub key: Key,
    pub modifiers: Modifiers,
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
        Self::from_char(value)
    }
}

impl From<Key> for Keybind {
    fn from(value: Key) -> Self {
        Self::from_key(value)
    }
}
