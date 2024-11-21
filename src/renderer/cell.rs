use compact_str::{CompactString, ToCompactString};
use unicode_width::{UnicodeWidthChar, UnicodeWidthStr};

use crate::{renderer::Rgba, Str};

#[derive(Clone, PartialEq, Eq, Debug)]
pub enum Cell {
    Grapheme(Grapheme),
    Pixel(Pixel),
    Empty,
    Continuation,
}

impl Cell {
    /// Set the foreground color of this cell
    pub fn set_fg(&mut self, fg: impl Into<Color>) {
        match self {
            Self::Grapheme(grapheme) => grapheme.fg = fg.into(),
            Self::Pixel(pixel) => pixel.fg = fg.into(),
            _ => {}
        }
    }

    /// Set the background color of this cell
    pub fn set_bg(&mut self, bg: impl Into<Color>) {
        match self {
            Self::Grapheme(grapheme) => grapheme.bg = bg.into(),
            Self::Pixel(pixel) => pixel.bg = bg.into(),
            _ => {}
        }
    }

    /// Appends this attribute to the current attributes of the cell
    pub fn set_attribute(&mut self, attribute: Attribute) {
        match self {
            Self::Grapheme(grapheme) => grapheme.attribute |= attribute,
            Self::Pixel(pixel) => pixel.attribute |= attribute,
            _ => {}
        }
    }

    /// Gets the foreground color mode for this cell
    pub const fn fg(&self) -> Color {
        match self {
            Self::Grapheme(grapheme) => grapheme.fg,
            Self::Pixel(pixel) => pixel.fg,
            _ => unreachable!(),
        }
    }

    /// Gets the background color mode for this cell
    pub const fn bg(&self) -> Color {
        match self {
            Self::Grapheme(grapheme) => grapheme.bg,
            Self::Pixel(pixel) => pixel.bg,
            _ => unreachable!(),
        }
    }

    /// Gets the attributes currently set for this cell
    pub const fn attribute(&self) -> Attribute {
        match self {
            Self::Grapheme(grapheme) => grapheme.attribute,
            Self::Pixel(pixel) => pixel.attribute,
            _ => unreachable!(),
        }
    }
}

impl Cell {
    pub(crate) fn is_same(&self, other: &Self) -> bool {
        fn check(fg: Color, bg: Color) -> bool {
            matches!(bg, Color::Reuse) || matches!(fg, Color::Reuse)
        }

        match (self, other) {
            (Cell::Grapheme(left), Cell::Grapheme(right)) => {
                (left == right) || (left.cluster == right.cluster && check(right.bg, right.fg))
            }
            (Cell::Grapheme(left), Cell::Pixel(right)) => {
                compare(&left.cluster, right.char) && check(right.bg, right.fg)
            }
            (Cell::Pixel(left), Cell::Grapheme(right)) => {
                compare(&right.cluster, left.char) && check(right.bg, right.fg)
            }
            (Cell::Pixel(left), Cell::Pixel(right)) => {
                (left == right) || ((left.char == right.char) && check(right.bg, right.fg))
            }
            (Cell::Empty, Cell::Grapheme(..) | Cell::Pixel(..)) => false,
            (Cell::Empty, Cell::Continuation | Cell::Empty) => true,
            _ => false,
        }
    }

    pub(crate) fn merge(mut this: &mut Self, other: Self) {
        fn merge_fg(left_fg: &mut Color, right_fg: Color) {
            if let (Color::Reset | Color::Set(..), ..) = (right_fg, &left_fg) {
                *left_fg = right_fg
            }
        }

        fn merge_bg(left_bg: &mut Color, right_bg: Color) {
            match (right_bg, &left_bg) {
                (Color::Set(a), Color::Set(b)) => *left_bg = Color::Set(a.blend_alpha(*b)),
                (Color::Reset | Color::Set(..), ..) => *left_bg = right_bg,
                _ => {}
            }
        }

        match (&mut this, other) {
            (Cell::Grapheme(ref mut left), Cell::Grapheme(mut right)) => {
                merge_fg(&mut left.fg, right.fg);
                merge_bg(&mut left.bg, right.bg);
                left.attribute = right.attribute;
                left.cluster = std::mem::take(&mut right.cluster);
            }
            (Cell::Grapheme(ref mut left), Cell::Pixel(right)) => {
                merge_fg(&mut left.fg, right.fg);
                merge_bg(&mut left.bg, right.bg);
                let pixel = Pixel {
                    char: right.char,
                    fg: left.fg,
                    bg: left.bg,
                    attribute: right.attribute,
                };
                *this = Cell::Pixel(pixel)
            }
            (Cell::Pixel(ref mut left), Cell::Grapheme(mut right)) => {
                merge_fg(&mut left.fg, right.fg);
                merge_bg(&mut left.bg, right.bg);
                let grapheme = Grapheme {
                    cluster: std::mem::take(&mut right.cluster),
                    fg: left.fg,
                    bg: left.bg,
                    attribute: right.attribute,
                };
                *this = Cell::Grapheme(grapheme)
            }

            (Cell::Pixel(ref mut left), Cell::Pixel(right)) => {
                merge_fg(&mut left.fg, right.fg);
                merge_bg(&mut left.bg, right.bg);
                left.attribute = right.attribute;
                left.char = right.char;
            }

            (_, right @ (Cell::Grapheme(..) | Cell::Pixel(..))) => *this = right,
            _ => {}
        }
    }

    pub(crate) fn width(&self) -> usize {
        match self {
            Self::Grapheme(grapheme) => grapheme.cluster.width(),
            Self::Pixel(pixel) => pixel.char.width().unwrap_or(0),
            Self::Empty | Self::Continuation => 0,
        }
    }

    pub(crate) const fn is_continuation(&self) -> bool {
        matches!(self, Self::Continuation)
    }

    pub(crate) const fn is_empty(&self) -> bool {
        matches!(self, Self::Empty)
    }
}

impl Default for Cell {
    fn default() -> Self {
        Self::Empty
    }
}

// impl std::fmt::Debug for Cell {
//     fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
//         match self {
//             Self::Grapheme(grapheme) => f.debug_tuple("Grapheme").field(&grapheme.cluster).finish(),
//             Self::Pixel(pixel) => f.debug_tuple("Pixel").field(&pixel.char).finish(),
//             Self::Empty => write!(f, "Empty"),
//             Self::Continuation => write!(f, "Continuation"),
//         }
//     }
// }

impl From<Grapheme> for Cell {
    fn from(value: Grapheme) -> Self {
        Self::Grapheme(value)
    }
}

impl From<Pixel> for Cell {
    fn from(value: Pixel) -> Self {
        Self::Pixel(value)
    }
}

impl From<Rgba> for Cell {
    fn from(bg: Rgba) -> Self {
        Cell::Pixel(Pixel::from(bg))
    }
}

impl<T: ToCompactString> From<T> for Cell {
    fn from(value: T) -> Self {
        Self::Grapheme(Grapheme::from(value))
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct Pixel {
    pub(crate) char: char,
    pub(crate) fg: Color,
    pub(crate) bg: Color,
    attribute: Attribute,
}

impl Default for Pixel {
    fn default() -> Self {
        Self::DEFAULT
    }
}

impl From<Rgba> for Pixel {
    fn from(value: Rgba) -> Self {
        Self::new(' ').bg(value)
    }
}

pub(crate) fn compare(left: &str, right: char) -> bool {
    let mut b: [u8; 4] = [0; 4];
    left == right.encode_utf8(&mut b)
}

impl Pixel {
    pub(crate) const DEFAULT: Self = Self {
        char: ' ',
        fg: Color::Reset,
        bg: Color::Reset,
        attribute: Attribute::RESET,
    };

    pub const fn new(char: char) -> Self {
        Self {
            char,
            fg: Color::Reset,
            bg: Color::Reuse,
            attribute: Attribute::RESET,
        }
    }

    pub const fn char(mut self, char: char) -> Self {
        self.char = char;
        self
    }

    pub fn fg(mut self, fg: impl Into<Color>) -> Self {
        self.fg = fg.into();
        self
    }

    pub fn bg(mut self, bg: impl Into<Color>) -> Self {
        self.bg = bg.into();
        self
    }

    pub fn attribute(mut self, attribute: Attribute) -> Self {
        self.attribute = attribute;
        self
    }
}

impl From<char> for Pixel {
    fn from(char: char) -> Self {
        Self::new(char)
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Grapheme {
    pub(crate) cluster: CompactString,
    pub(crate) fg: Color,
    pub(crate) bg: Color,
    attribute: Attribute,
}

impl Grapheme {
    pub const fn const_new(str: &'static str) -> Self {
        Self {
            cluster: CompactString::const_new(str),
            fg: Color::Reset,
            bg: Color::Reuse,
            attribute: Attribute::RESET,
        }
    }

    pub fn new(data: impl Into<Str>) -> Self {
        Self {
            cluster: data.into().into_inner(),
            fg: Color::Reset,
            bg: Color::Reuse,
            attribute: Attribute::RESET,
        }
    }

    pub fn fg(mut self, fg: impl Into<Color>) -> Self {
        self.fg = fg.into();
        self
    }

    pub fn bg(mut self, bg: impl Into<Color>) -> Self {
        self.bg = bg.into();
        self
    }

    pub fn attribute(mut self, attribute: Attribute) -> Self {
        self.attribute = attribute;
        self
    }
}

impl<T: ToCompactString> From<T> for Grapheme {
    fn from(value: T) -> Self {
        Self::new(value)
    }
}

// #[derive(Copy, Clone, Debug, PartialEq, PartialOrd, Eq, Ord, Hash)]
// pub struct Underline(pub(crate) u8);
// impl Underline {
//     // <ESC>[4:0m  # no underline
//     // <ESC>[4:1m  # straight underline
//     // <ESC>[4:2m  # double underline
//     // <ESC>[4:3m  # curly underline
//     // <ESC>[4:4m  # dotted underline
//     // <ESC>[4:5m  # dashed underline

//     pub const NONE: Self = Self(0);
//     pub const STRAIGHT: Self = Self(1 << 0);
//     pub const DOUBLE: Self = Self(1 << 1);
//     pub const CURLY: Self = Self(1 << 2);
//     pub const DOTTED: Self = Self(1 << 3);
//     pub const DASHED: Self = Self(1 << 4);
// }

/// Attributes for a [`Pixel`] like _italic_ or _bold_
#[derive(Copy, Clone, Default, PartialEq, PartialOrd, Eq, Ord, Hash)]
pub struct Attribute(pub u16);

impl Attribute {
    pub const RESET: Self = Self(0);
    pub const BOLD: Self = Self(1 << 0);
    pub const FAINT: Self = Self(1 << 1);
    pub const ITALIC: Self = Self(1 << 2);
    pub const UNDERLINE: Self = Self(1 << 3);
    pub const BLINK: Self = Self(1 << 4);
    pub const REVERSE: Self = Self(1 << 6);
    pub const STRIKEOUT: Self = Self(1 << 8);
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
        self.0 & (1 << 6) != 0
    }

    pub const fn is_strikeout(&self) -> bool {
        self.0 & (1 << 8) != 0
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
        const FIELDS: [&str; 9] = [
            "Bold",
            "Faint",
            "Italic",
            "Underline",
            "Blink",
            "", // rapid blink
            "Reverse",
            "", // conceal
            "Strikeout",
        ];

        let mut seen = false;
        for (flag, repr) in (0..).zip(FIELDS) {
            if repr.is_empty() {
                continue;
            }

            if (self.0 >> flag) & 1 == 1 {
                if seen {
                    f.write_str(" | ")?;
                }
                f.write_str(repr)?;
                seen |= true
            }
        }

        if !seen || self.0 == 0 {
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

/// Color mode for a Pixel
#[derive(Copy, Clone, Default, Debug, PartialEq, Eq)]
pub enum Color {
    /// Use this color
    Set(Rgba),
    #[default]
    /// Reuse the existing color
    Reuse,
    /// Reset the default color
    Reset,
}

impl Color {
    /// Overrides the current color mode with it set to a specific [`Rgba`]
    pub fn set_color(&mut self, rgba: impl Into<Rgba>) {
        *self = Self::Set(rgba.into())
    }

    /// If this color is set, returns it
    pub fn as_rgba(&self) -> Option<Rgba> {
        match self {
            Self::Set(rgba) => Some(*rgba),
            _ => None,
        }
    }
}

impl From<&'static str> for Color {
    fn from(value: &'static str) -> Self {
        Self::Set(Rgba::hex(value))
    }
}

impl From<Rgba> for Color {
    fn from(value: Rgba) -> Self {
        Self::Set(value)
    }
}

impl<T: Into<Rgba>> From<Option<T>> for Color {
    fn from(value: Option<T>) -> Self {
        value.map_or(Self::Reset, |val| Self::Set(val.into()))
    }
}
