/// Interests for events that a view want to recieve
#[derive(Copy, Clone, Default, PartialEq)]
pub struct Interest(u8);

impl std::fmt::Debug for Interest {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        const FIELDS: [&str; 6] = [
            "MOUSE_INSIDE",
            "MOUSE_OUTSIDE",
            "MOUSE_MOVE",
            "FOCUS",
            "FOCUS_INPUT",
            "SELECTION_CHANGE",
        ];

        let mut seen = false;
        for (flag, repr) in (0..).zip(FIELDS) {
            if (self.0 >> flag) & 1 == 1 {
                if seen {
                    f.write_str(" | ")?;
                }
                f.write_str(repr)?;
                seen |= true
            }
        }

        if !seen {
            f.write_str("NONE")?;
        }

        Ok(())
    }
}

impl Interest {
    /// No events should be sent to a view
    pub const NONE: Self = Self(0);
    /// All events should be sent to a view
    pub const ALL: Self = Self(u8::MAX);

    /// A view wants to get mouse events inside of it
    pub const MOUSE_INSIDE: Self = Self(1 << 0);
    /// A view wants to get mouse events outside of it
    pub const MOUSE_OUTSIDE: Self = Self(1 << 1);
    /// A view wants to get mouse move events
    pub const MOUSE_MOVE: Self = Self(1 << 2);

    /// A view wants to get [`ViewEvent::FocusGained`](crate::view::ViewEvent::FocusGained) and [`ViewEvent::FocusLost`](crate::view::ViewEvent::FocusLost) events
    pub const FOCUS: Self = Self(1 << 3);

    /// A view wants to key inputs
    pub const FOCUS_INPUT: Self = Self(1 << 4);

    /// A view wants to get [`ViewEvent::SelectionAdded`](crate::view::ViewEvent::SelectionAdded) and [`ViewEvent::SelectionRemoved`](crate::view::ViewEvent::SelectionRemoved) events
    pub const SELECTION_CHANGE: Self = Self(1 << 5);

    /// A view wants all mouse events
    pub const MOUSE: Self = Self(1 << 0 | 1 << 1 | 1 << 2);
}

impl Interest {
    pub const fn is_none(&self) -> bool {
        self.0 == 0
    }

    pub const fn is_mouse_inside(&self) -> bool {
        self.0 & (1 << 0) != 0
    }

    pub const fn is_mouse_outside(&self) -> bool {
        self.0 & (1 << 1) != 0
    }

    pub const fn is_mouse_move(&self) -> bool {
        self.0 & (1 << 2) != 0
    }

    pub const fn is_mouse_any(&self) -> bool {
        self.is_mouse_inside() || self.is_mouse_outside() || self.is_mouse_move()
    }

    pub const fn is_focus(&self) -> bool {
        self.0 & (1 << 3) != 0
    }

    pub const fn is_focus_input(&self) -> bool {
        self.0 & (1 << 4) != 0
    }

    pub const fn is_selection_change(&self) -> bool {
        self.0 & (1 << 5) != 0
    }
}

impl std::ops::BitAnd for Interest {
    type Output = Self;
    fn bitand(self, rhs: Self) -> Self::Output {
        Self(self.0 & rhs.0)
    }
}

impl std::ops::BitOr for Interest {
    type Output = Self;
    fn bitor(self, rhs: Self) -> Self::Output {
        Self(self.0 | rhs.0)
    }
}

impl std::ops::BitXor for Interest {
    type Output = Self;
    fn bitxor(self, rhs: Self) -> Self::Output {
        Self(self.0 ^ rhs.0)
    }
}

impl std::ops::BitAndAssign for Interest {
    fn bitand_assign(&mut self, rhs: Self) {
        *self = *self & rhs
    }
}

impl std::ops::BitOrAssign for Interest {
    fn bitor_assign(&mut self, rhs: Self) {
        *self = *self | rhs
    }
}

impl std::ops::BitXorAssign for Interest {
    fn bitxor_assign(&mut self, rhs: Self) {
        *self = *self ^ rhs
    }
}

impl std::ops::Not for Interest {
    type Output = Self;
    fn not(self) -> Self::Output {
        Self(self.0)
    }
}
