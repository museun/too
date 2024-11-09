use crate::math::Margin;

// TODO move this out into the main module
#[derive(Copy, Clone, PartialEq)]
pub struct Border {
    pub left_top: char,
    pub top: char,
    pub right_top: char,
    pub right: char,
    pub right_bottom: char,
    pub bottom: char,
    pub left_bottom: char,
    pub left: char,
}

impl std::fmt::Debug for Border {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Border")
            .field("left_top", &format_args!("0x{:04X}", self.left_top as u32))
            .field("top", &format_args!("0x{:04X}", self.top as u32))
            .field(
                "right_top",
                &format_args!("0x{:04X}", self.right_top as u32),
            )
            .field("right", &format_args!("0x{:04X}", self.right as u32))
            .field(
                "right_bottom",
                &format_args!("0x{:04X}", self.right_bottom as u32),
            )
            .field("bottom", &format_args!("0x{:04X}", self.bottom as u32))
            .field(
                "left_bottom",
                &format_args!("0x{:04X}", self.left_bottom as u32),
            )
            .field("left", &format_args!("0x{:04X}", self.left as u32))
            .finish()
    }
}

impl Border {
    pub const fn without_top(self) -> Self {
        Self {
            left_top: self.left,
            right_top: self.right,
            top: ' ',
            ..self
        }
    }

    pub const fn without_bottom(self) -> Self {
        Self {
            left_bottom: self.left,
            right_bottom: self.right,
            bottom: ' ',
            ..self
        }
    }

    pub const fn without_left(self) -> Self {
        Self {
            left_top: self.top,
            left: ' ',
            left_bottom: self.bottom,
            ..self
        }
    }

    pub const fn without_right(self) -> Self {
        Self {
            right_top: self.top,
            right: ' ',
            right_bottom: self.bottom,
            ..self
        }
    }

    pub const fn without_corners(self) -> Self {
        Self {
            left_top: self.top,
            right_top: self.top,
            left_bottom: self.bottom,
            right_bottom: self.bottom,
            ..self
        }
    }

    const fn has(c: char) -> bool {
        c != ' '
    }

    pub const fn has_left(&self) -> bool {
        Self::has(self.left) | (Self::has(self.left_top) && Self::has(self.left_bottom))
    }

    pub const fn has_top(&self) -> bool {
        Self::has(self.top) | (Self::has(self.left_top) && Self::has(self.right_top))
    }

    pub const fn has_right(&self) -> bool {
        Self::has(self.right) | (Self::has(self.right_top) && Self::has(self.right_bottom))
    }

    pub const fn has_bottom(&self) -> bool {
        Self::has(self.bottom) | (Self::has(self.left_bottom) && Self::has(self.right_bottom))
    }

    pub fn as_margin(&self) -> Margin {
        Margin::new(
            self.has_left() as i32,
            self.has_top() as i32,
            self.has_right() as i32,
            self.has_bottom() as i32,
        )
    }
}

impl Default for Border {
    fn default() -> Self {
        Self::THICK
    }
}

impl Border {
    pub const EMPTY: Self = Self {
        left_top: ' ',
        top: ' ',
        right_top: ' ',
        right: ' ',
        right_bottom: ' ',
        bottom: ' ',
        left_bottom: ' ',
        left: ' ',
    };

    pub const THIN: Self = Self {
        left_top: '┌',
        top: '─',
        right_top: '┐',
        right: '│',
        right_bottom: '┘',
        bottom: '─',
        left_bottom: '└',
        left: '│',
    };

    pub const THIN_WIDE: Self = Self {
        left_top: '▁',
        top: '▁',
        right_top: '▁',
        right: '▕',
        right_bottom: '▔',
        bottom: '▔',
        left_bottom: '▔',
        left: '▏',
    };

    pub const ROUNDED: Self = Self {
        left_top: '╭',
        top: '─',
        right_top: '╮',
        right: '│',
        right_bottom: '╯',
        bottom: '─',
        left_bottom: '╰',
        left: '│',
    };

    pub const DOUBLE: Self = Self {
        left_top: '╔',
        top: '═',
        right_top: '╗',
        right: '║',
        right_bottom: '╝',
        bottom: '═',
        left_bottom: '╚',
        left: '║',
    };

    pub const THICK: Self = Self {
        left_top: '┏',
        top: '━',
        right_top: '┓',
        right: '┃',
        right_bottom: '┛',
        bottom: '━',
        left_bottom: '┗',
        left: '┃',
    };

    pub const THICK_TALL: Self = Self {
        left_top: '▛',
        top: '▀',
        right_top: '▜',
        right: '▐',
        right_bottom: '▟',
        bottom: '▄',
        left_bottom: '▙',
        left: '▌',
    };

    pub const THICK_WIDE: Self = Self {
        left_top: '▗',
        top: '▄',
        right_top: '▖',
        right: '▌',
        right_bottom: '▘',
        bottom: '▀',
        left_bottom: '▝',
        left: '▐',
    };
}
