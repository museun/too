#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum Key {
    Char(char),
    Function(u8),
    Left,
    Right,
    Up,
    Down,
    PageUp,
    PageDown,
    Home,
    End,
    Insert,
    Enter,
    Delete,
    Backspace,
    Escape,
    Tab,
}
