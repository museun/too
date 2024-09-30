use crate::Rgba;

/// A unit to be drawn to a [`Surface`](crate::Surface)
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct Pixel {
    pub char: char, // this has to be a &str
    pub fg: PixelColor,
    pub bg: PixelColor,
    pub attr: Attribute,
}

impl Pixel {
    pub const EMPTY: Self = Self {
        char: ' ',
        fg: PixelColor::Reset,
        bg: PixelColor::Reset,
        attr: Attribute::RESET,
    };

    pub const fn new(char: char) -> Self {
        Self {
            char,
            fg: PixelColor::Reset,
            bg: PixelColor::Reuse,
            attr: Attribute::RESET,
        }
    }

    pub const fn char(mut self, char: char) -> Self {
        self.char = char;
        self
    }

    pub fn fg(mut self, fg: impl Into<PixelColor>) -> Self {
        self.fg = fg.into();
        self
    }

    pub fn bg(mut self, bg: impl Into<PixelColor>) -> Self {
        self.bg = bg.into();
        self
    }

    pub fn attr(mut self, attr: impl Into<Option<Attribute>>) -> Self {
        self.attr = attr.into().unwrap_or(Attribute::RESET);
        self
    }

    pub fn update(&mut self, mut update: impl FnMut(&mut Self)) {
        update(self)
    }

    pub fn merge(&mut self, other: Self) {
        match (other.bg, self.bg) {
            (PixelColor::Rgba(a), PixelColor::Rgba(b)) => {
                self.bg = PixelColor::Rgba(a.blend_alpha(b))
            }
            (PixelColor::Reset | PixelColor::Rgba(..), ..) => self.bg = other.bg,
            _ => {}
        }

        if let (PixelColor::Reset | PixelColor::Rgba(..), ..) = (other.fg, self.fg) {
            self.fg = other.fg
        }

        self.char = other.char;
        self.attr = other.attr;
    }
}

impl From<char> for Pixel {
    fn from(value: char) -> Self {
        Self::new(value)
    }
}

impl Default for Pixel {
    fn default() -> Self {
        Self::EMPTY
    }
}

/// Attributes for a [`Pixel`] like _italic_ or _bold_
#[derive(Copy, Clone, Default, PartialEq, PartialOrd, Eq, Ord, Hash)]
pub struct Attribute(u8);

impl Attribute {
    pub const RESET: Self = Self(0);
    pub const BOLD: Self = Self(1 << 0);
    pub const FAINT: Self = Self(1 << 1);
    pub const ITALIC: Self = Self(1 << 2);
    pub const UNDERLINE: Self = Self(1 << 3);
    pub const BLINK: Self = Self(1 << 4);
    pub const REVERSE: Self = Self(1 << 5);
    pub const STRIKEOUT: Self = Self(1 << 6);
}

impl Attribute {
    pub const fn is_reset(&self) -> bool {
        self.0 == 0
    }

    pub const fn is_bold(&self) -> bool {
        self.0 & (1 << 0) != 0
    }

    pub const fn is_faint(&self) -> bool {
        self.0 & (1 << 1) != 0
    }

    pub const fn is_italic(&self) -> bool {
        self.0 & (1 << 2) != 0
    }

    pub const fn is_underline(&self) -> bool {
        self.0 & (1 << 3) != 0
    }

    pub const fn is_blink(&self) -> bool {
        self.0 & (1 << 4) != 0
    }

    pub const fn is_reverse(&self) -> bool {
        self.0 & (1 << 5) != 0
    }

    pub const fn is_strikeout(&self) -> bool {
        self.0 & (1 << 6) != 0
    }
}

impl std::ops::BitAnd for Attribute {
    type Output = Self;
    fn bitand(self, rhs: Self) -> Self::Output {
        Self(self.0 & rhs.0)
    }
}
impl std::ops::BitAndAssign for Attribute {
    fn bitand_assign(&mut self, rhs: Self) {
        *self = *self & rhs
    }
}

impl std::ops::BitOr for Attribute {
    type Output = Self;
    fn bitor(self, rhs: Self) -> Self::Output {
        Self(self.0 | rhs.0)
    }
}
impl std::ops::BitOrAssign for Attribute {
    fn bitor_assign(&mut self, rhs: Self) {
        *self = *self | rhs
    }
}

impl std::ops::Not for Attribute {
    type Output = Self;
    fn not(self) -> Self::Output {
        Self(!self.0)
    }
}

impl std::fmt::Debug for Attribute {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        const FIELDS: [&str; 7] = [
            "Bold",
            "Faint",
            "Italic",
            "Underline",
            "Blink",
            "Reverse",
            "Strikeout",
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
            f.write_str("Reset")?;
        }

        Ok(())
    }
}

impl std::str::FromStr for Attribute {
    type Err = String;
    fn from_str(input: &str) -> Result<Self, Self::Err> {
        let mut this = Self::RESET;
        for part in input.split_terminator('+').map(<str>::trim) {
            this |= match part {
                s if s.eq_ignore_ascii_case("bold") => Self::BOLD,
                s if s.eq_ignore_ascii_case("faint") => Self::FAINT,
                s if s.eq_ignore_ascii_case("italic") => Self::ITALIC,
                s if s.eq_ignore_ascii_case("underline") => Self::UNDERLINE,
                s if s.eq_ignore_ascii_case("blink") => Self::BLINK,
                s if s.eq_ignore_ascii_case("reverse") => Self::REVERSE,
                s if s.eq_ignore_ascii_case("strikeout") => Self::STRIKEOUT,
                attr => return Err(format!("unknown attribute: {attr}")),
            }
        }
        Ok(this)
    }
}

// TODO rename this

/// Color mode for a Pixel
#[derive(Copy, Clone, Default, Debug, PartialEq, Eq)]
pub enum PixelColor {
    /// Use this color
    Rgba(Rgba),
    #[default]
    /// Reuse the existing color
    Reuse,
    /// Reset the default color
    Reset,
}

impl From<&'static str> for PixelColor {
    fn from(value: &'static str) -> Self {
        Self::Rgba(Rgba::hex(value))
    }
}

impl From<Rgba> for PixelColor {
    fn from(value: Rgba) -> Self {
        Self::Rgba(value)
    }
}

impl<T: Into<Rgba>> From<Option<T>> for PixelColor {
    fn from(value: Option<T>) -> Self {
        value.map_or(Self::Reset, |val| Self::Rgba(val.into()))
    }
}
