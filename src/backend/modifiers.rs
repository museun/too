#[derive(Copy, Clone, Default, PartialEq, PartialOrd, Eq, Ord, Hash)]
pub struct Modifiers(pub u8);

impl Modifiers {
    pub const NONE: Self = Self(0);
    pub const SHIFT: Self = Self(1 << 0);
    pub const CTRL: Self = Self(1 << 1);
    pub const ALT: Self = Self(1 << 2);
}

impl Modifiers {
    pub const fn is_none(&self) -> bool {
        self.0 == 0
    }

    pub const fn is_shift(&self) -> bool {
        self.0 & 1 == 1
    }

    pub const fn is_ctrl(&self) -> bool {
        (self.0 >> 1) & 1 == 1
    }

    pub const fn is_alt(&self) -> bool {
        (self.0 >> 2) & 1 == 1
    }

    pub const fn is_shift_only(&self) -> bool {
        self.0 == Self::SHIFT.0
    }

    pub const fn is_ctrl_only(&self) -> bool {
        self.0 == Self::CTRL.0
    }

    pub const fn is_alt_only(&self) -> bool {
        self.0 == Self::ALT.0
    }

    pub const fn remove(&self, other: Self) -> Self {
        Self(self.0 & !other.0)
    }
}

impl std::ops::BitAnd for Modifiers {
    type Output = Self;
    fn bitand(self, rhs: Self) -> Self::Output {
        Self(self.0 & rhs.0)
    }
}
impl std::ops::BitAndAssign for Modifiers {
    fn bitand_assign(&mut self, rhs: Self) {
        *self = *self & rhs
    }
}

impl std::ops::BitOr for Modifiers {
    type Output = Self;
    fn bitor(self, rhs: Self) -> Self::Output {
        Self(self.0 | rhs.0)
    }
}
impl std::ops::BitOrAssign for Modifiers {
    fn bitor_assign(&mut self, rhs: Self) {
        *self = *self | rhs
    }
}

impl std::ops::Not for Modifiers {
    type Output = Self;
    fn not(self) -> Self::Output {
        Self(!self.0)
    }
}

impl std::fmt::Debug for Modifiers {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        const FIELDS: [&str; 3] = ["SHIFT", "CTRL", "ALT"];

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

impl std::str::FromStr for Modifiers {
    type Err = String;
    fn from_str(input: &str) -> Result<Self, Self::Err> {
        let mut this = Self::NONE;
        for part in input.split_terminator('+').map(<str>::trim) {
            this |= match part {
                s if s.eq_ignore_ascii_case("shift") => Self::SHIFT,
                s if s.eq_ignore_ascii_case("ctrl") => Self::CTRL,
                s if s.eq_ignore_ascii_case("alt") => Self::ALT,
                attr => return Err(format!("unknown modifier: {attr}")),
            }
        }
        Ok(this)
    }
}
