// TODO Axis support for this
pub struct Elements;
impl Elements {
    pub const LARGE_RECT: char = '█';
    pub const MEDIUM_RECT: char = '■';
    pub const SMALL_RECT: char = '▮';

    pub const CIRCLE: char = '●';
    pub const DIAMOND: char = '◆';

    pub const HORIZONTAL_LINE: char = '─';
    pub const THICK_HORIZONTAL_LINE: char = '━';
    pub const DASH_HORIZONTAL_LINE: char = '╌';
    pub const THICK_DASH_HORIZONTAL_LINE: char = '╍';

    pub const VERTICAL_LINE: char = '│';
    pub const THICK_VERTICAL_LINE: char = '┃';
    pub const DASH_VERTICAL_LINE: char = '╎';
    pub const THICK_DASH_VERTICAL_LINE: char = '╏';
}

pub struct Knob(pub char);
impl Default for Knob {
    fn default() -> Self {
        Self::ROUND
    }
}

impl Knob {
    pub const LARGE: Self = Self(Elements::LARGE_RECT);
    pub const MEDIUM: Self = Self(Elements::MEDIUM_RECT);
    pub const SMALL: Self = Self(Elements::SMALL_RECT);
    pub const ROUND: Self = Self(Elements::CIRCLE);
    pub const DIAMOND: Self = Self(Elements::DIAMOND);
}
