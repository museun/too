use crate::{
    backend::{Key, Modifiers, MouseButton},
    math::{Pos2, Vec2},
    view::ViewId,
};

/// Events a view can receive
#[derive(Copy, Clone, Debug, PartialEq)]
pub enum ViewEvent {
    /// A key was pressed
    KeyInput {
        /// The key code that was pressed
        key: Key,
        /// Any modifiers being held down while the key was pressed
        modifiers: Modifiers,
    },
    /// The mouse was moved
    MouseMove {
        /// The current position of the mouse cursor
        pos: Pos2,
        /// Any modifiers being held down while the mouse was being moved
        modifiers: Modifiers,
    },
    /// A mouse button was held down
    MouseHeld {
        /// The current position of the mouse cursor
        pos: Pos2,
        /// Was it inside of the view?
        inside: bool,
        /// The button that is being held down
        button: MouseButton,
        /// Any modifiers being held down while the mouse button was also being held down
        modifiers: Modifiers,
    },
    /// A mouse button was pressed then released
    MouseClicked {
        /// The current position of the mouse cursor
        pos: Pos2,
        /// Was it inside of the view?
        inside: bool,
        /// The button that was clicked
        button: MouseButton,
        /// Any modifiers being held down while the mouse button was clicked
        modifiers: Modifiers,
    },
    /// A mouse button was being held down and then moved
    MouseDrag {
        /// The start position of the drag
        start: Pos2,
        /// The current position of the mouse cursor
        current: Pos2,
        /// The delta from the last time this event was sent
        ///
        /// - Positive
        /// - - Down or Right
        /// - Negative
        /// - - Up or Left
        delta: Vec2,
        /// Was it inside of the view?
        inside: bool,
        /// The button that is being held down
        modifiers: Modifiers,
        /// Any modifiers being held down while the mouse button was also being held down
        button: MouseButton,
    },
    /// The mouse was scrolled
    MouseScroll {
        /// The delta from the last time this event was sent
        ///
        /// - Positive
        /// - - Down or Right
        /// - Negative
        /// - - Up or Left
        delta: Vec2,
        /// Any modifiers being held down while the mouse was scrolling
        modifiers: Modifiers,
    },

    /// The mouse cursor entered the view
    MouseEntered,
    /// The mouse cursor left the view
    MouseLeave,

    /// The view gained focus
    FocusGained,
    /// The view lost focus
    FocusLost,

    /// A view notified that it was selected
    SelectionAdded(ViewId),
    /// A view notified that it was unselected
    SelectionRemoved(ViewId),
}
