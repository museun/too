/// Which screen the backend is on
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum CurrentScreen {
    /// The main screen (the one that doesn't get drawn too)
    Main,
    /// The alt screen (the one that does get drawn too)
    Alt,
}

impl CurrentScreen {
    /// Is this the main screen?
    pub const fn is_main_screen(&self) -> bool {
        matches!(self, Self::Main)
    }

    /// Is this the alt screen?
    pub const fn is_alt_screen(&self) -> bool {
        matches!(self, Self::Alt)
    }

    /// Toggle to the other screen
    pub fn toggle(self) -> Self {
        match self {
            Self::Main => Self::Alt,
            Self::Alt => Self::Main,
        }
    }
}
