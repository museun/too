#[derive(Copy, Clone, Debug, PartialEq, PartialOrd)]
pub enum Flex {
    Tight(f32),
    Loose(f32),
}

impl Flex {
    pub const fn factor(&self) -> f32 {
        match self {
            Self::Tight(factor) | Self::Loose(factor) => *factor,
        }
    }

    pub const fn has_flex(&self) -> bool {
        self.factor() != 0.0
    }

    pub const fn is_expand(&self) -> bool {
        matches!(self, Self::Tight(..))
    }
}

impl From<i32> for Flex {
    fn from(value: i32) -> Self {
        Self::Tight(value as f32)
    }
}

impl From<f32> for Flex {
    fn from(value: f32) -> Self {
        Self::Tight(value)
    }
}
