use crate::{
    math::{Pos2, Vec2},
    view::ViewId,
    Key, Modifiers, MouseButton,
};

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum ViewEvent {
    KeyInput {
        key: Key,
        modifiers: Modifiers,
    },
    MouseMove {
        pos: Pos2,
        modifiers: Modifiers,
    },
    MouseHeld {
        pos: Pos2,
        inside: bool,
        button: MouseButton,
        modifiers: Modifiers,
    },
    MouseClicked {
        pos: Pos2,
        inside: bool,
        button: MouseButton,
        modifiers: Modifiers,
    },
    MouseDrag {
        start: Pos2,
        current: Pos2,
        delta: Vec2,
        inside: bool,
        modifiers: Modifiers,
        button: MouseButton,
    },
    MouseScroll {
        delta: Vec2,
        modifiers: Modifiers,
    },
    MouseEntered,
    MouseLeave,

    FocusGained,
    FocusLost,

    SelectionAdded(ViewId),
    SelectionRemoved(ViewId),
}
