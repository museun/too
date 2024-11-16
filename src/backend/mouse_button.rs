/// A mouse button
#[derive(Copy, Clone, Debug, Default, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum MouseButton {
    #[default]
    // The primary button (typically, left)
    Primary,
    // The secondary button (typically, right)
    Secondary,
    /// The middle button (typically, the scroll wheel)
    Middle,
}

impl MouseButton {
    // Is this the primary button? (typically, left)
    pub const fn is_primary(&self) -> bool {
        matches!(self, Self::Primary)
    }

    // Is this the secondary button? (typically, right)
    pub const fn is_secondary(&self) -> bool {
        matches!(self, Self::Secondary)
    }

    /// Is this the middle button? (typically, the scroll wheel)
    pub const fn is_middle(&self) -> bool {
        matches!(self, Self::Middle)
    }
}
