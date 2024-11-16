#[derive(Copy, Clone, Debug, Default, PartialEq)]
pub enum CrossAlign {
    #[default]
    Start,
    End,
    Center,
    Stretch,
    Fill,
}

impl CrossAlign {
    pub const fn is_stretch(&self) -> bool {
        matches!(self, Self::Stretch)
    }

    pub const fn is_fill(&self) -> bool {
        matches!(self, Self::Fill)
    }

    pub fn align(self, available: f32, size: f32) -> f32 {
        match self {
            Self::Start | Self::Stretch | Self::Fill => 0.0,
            Self::End => available - size,
            Self::Center => (available - size) * 0.5,
        }
    }
}
