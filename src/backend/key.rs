/// A keyboard key
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum Key {
    /// A character key, like `s` or `@`
    Char(char),
    /// A function key, like F1 or F5
    Function(u8),
    /// Left arrow
    Left,
    /// Right arrow
    Right,
    /// Up arrow
    Up,
    /// Down arrow
    Down,
    /// Page up
    PageUp,
    /// Page down
    PageDown,
    /// Home
    Home,
    /// End
    End,
    /// Insert
    Insert,
    /// Enter
    Enter,
    /// Delete
    Delete,
    /// Backspace
    Backspace,
    /// Escape
    Escape,
    /// Tab
    Tab,
}

impl Key {
    pub const fn is_char(&self, ch: char) -> bool {
        if let &Self::Char(c) = self {
            return c == ch;
        }
        false
    }

    pub fn to_ascii_uppercase(&self) -> Option<Self> {
        let Self::Char(ch) = self else { return None };
        ch.is_ascii()
            .then(|| ch.to_ascii_uppercase())
            .map(Self::Char)
    }

    pub fn to_ascii_lowercase(&self) -> Option<Self> {
        let Self::Char(ch) = self else { return None };
        ch.is_ascii()
            .then(|| ch.to_ascii_lowercase())
            .map(Self::Char)
    }

    pub const fn is_ascii_uppercase(&self) -> bool {
        let Self::Char(ch) = self else { return false };
        ch.is_ascii_uppercase()
    }

    pub const fn is_ascii_lowercase(&self) -> bool {
        let Self::Char(ch) = self else { return false };
        ch.is_ascii_lowercase()
    }
}
