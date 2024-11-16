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
    pub const NONE: Self = Self(0);
    pub const ALL: Self = Self(u8::MAX);

    pub const MOUSE_INSIDE: Self = Self(1 << 0);
    pub const MOUSE_OUTSIDE: Self = Self(1 << 1);
    pub const MOUSE_MOVE: Self = Self(1 << 2);

    pub const FOCUS: Self = Self(1 << 3);
    pub const FOCUS_INPUT: Self = Self(1 << 4);

    pub const SELECTION_CHANGE: Self = Self(1 << 5);

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
