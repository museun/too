#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum CurrentScreen {
    Main,
    Alt,
}

impl CurrentScreen {
    pub const fn is_main_screen(&self) -> bool {
        matches!(self, Self::Main)
    }

    pub const fn is_alt_screen(&self) -> bool {
        matches!(self, Self::Alt)
    }

    pub fn toggle(self) -> Self {
        match self {
            Self::Main => Self::Alt,
            Self::Alt => Self::Main,
        }
    }
}
