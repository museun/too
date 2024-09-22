/// A keyboard key
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum Key {
    // A character key, like `s` or `@`
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
