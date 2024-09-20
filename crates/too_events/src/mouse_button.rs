#[derive(Copy, Clone, Debug, Default, PartialEq, Eq, Hash)]
pub enum MouseButton {
    #[default]
    Primary,
    Secondary,
    Middle,
}

impl MouseButton {
    pub const fn is_primary(&self) -> bool {
        matches!(self, Self::Primary)
    }

    pub const fn is_secondary(&self) -> bool {
        matches!(self, Self::Secondary)
    }

    pub const fn is_middle(&self) -> bool {
        matches!(self, Self::Middle)
    }
}
