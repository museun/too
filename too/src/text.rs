use std::borrow::Cow;

use unicode_segmentation::UnicodeSegmentation;
use unicode_width::{UnicodeWidthChar, UnicodeWidthStr};

use crate::{
    math::{pos2, vec2, Rect, Vec2},
    Attribute, Cell, Color, Grapheme, Pixel, Surface,
};

#[derive(Clone)]
pub enum GraphemeKind<'a> {
    Cluster(Cow<'a, str>),
    Scalar(char),
}

impl<'a> GraphemeKind<'a> {
    fn width(&self) -> usize {
        match self {
            Self::Cluster(c) => c.width(),
            Self::Scalar(s) => s.width().unwrap_or(0),
        }
    }
}

pub trait MeasureText {
    fn measure(&self) -> usize;
    fn graphemes(&self) -> impl Iterator<Item = GraphemeKind<'_>>;
}

impl<T: MeasureText> MeasureText for &T {
    fn measure(&self) -> usize {
        T::measure(*self)
    }

    fn graphemes(&self) -> impl Iterator<Item = GraphemeKind<'_>> {
        T::graphemes(*self)
    }
}

impl MeasureText for () {
    fn measure(&self) -> usize {
        0
    }

    fn graphemes(&self) -> impl Iterator<Item = GraphemeKind<'_>> {
        std::iter::empty()
    }
}

impl MeasureText for &str {
    fn measure(&self) -> usize {
        self.width()
    }

    fn graphemes(&self) -> impl Iterator<Item = GraphemeKind<'_>> {
        UnicodeSegmentation::graphemes(*self, true)
            .map(Cow::from)
            .map(GraphemeKind::Cluster)
    }
}

impl MeasureText for String {
    fn measure(&self) -> usize {
        self.width()
    }

    fn graphemes(&self) -> impl Iterator<Item = GraphemeKind<'_>> {
        UnicodeSegmentation::graphemes(self.as_str(), true)
            .map(Cow::from)
            .map(GraphemeKind::Cluster)
    }
}

impl MeasureText for char {
    fn measure(&self) -> usize {
        self.width().unwrap_or(0)
    }

    fn graphemes(&self) -> impl Iterator<Item = GraphemeKind<'_>> {
        std::iter::once(*self).map(GraphemeKind::Scalar)
    }
}

impl MeasureText for bool {
    fn measure(&self) -> usize {
        if *self {
            4
        } else {
            5
        }
    }

    fn graphemes(&self) -> impl Iterator<Item = GraphemeKind<'_>> {
        let mut s = Some(match self {
            true => "true",
            false => "false",
        });
        std::iter::from_fn(move || s.take())
            .map(Cow::from)
            .map(GraphemeKind::Cluster)
    }
}

impl MeasureText for i32 {
    fn measure(&self) -> usize {
        self::util::count_signed_digits(*self as _)
    }

    fn graphemes(&self) -> impl Iterator<Item = GraphemeKind<'_>> {
        self::util::signed_digits(*self as _).map(GraphemeKind::Scalar)
    }
}

impl MeasureText for usize {
    fn measure(&self) -> usize {
        self::util::count_digits(*self)
    }

    fn graphemes(&self) -> impl Iterator<Item = GraphemeKind<'_>> {
        self::util::digits(*self as _).map(GraphemeKind::Scalar)
    }
}

impl<T: MeasureText> MeasureText for Option<T> {
    fn measure(&self) -> usize {
        self.as_ref().map(T::measure).unwrap_or(0)
    }

    fn graphemes(&self) -> impl Iterator<Item = GraphemeKind<'_>> {
        let this = self.as_ref();
        std::iter::from_fn(move || {
            let this = this?;
            Some(this.graphemes())
        })
        .flatten()
    }
}

#[derive(Copy, Clone, Default, Debug, PartialEq)]
pub enum Justification {
    #[default]
    Start,
    Center,
    End,
}

pub struct Text<T: MeasureText> {
    pub text: T,
    fg: Color,
    bg: Color,
    attribute: Attribute,
    main: Justification,
    cross: Justification,
}

impl<T: MeasureText> From<T> for Text<T> {
    fn from(value: T) -> Self {
        Self::new(value)
    }
}

impl<T: MeasureText> Text<T> {
    pub const fn new(text: T) -> Self {
        Self {
            text,
            fg: Color::Reset,
            bg: Color::Reuse,
            attribute: Attribute::RESET,
            main: Justification::Start,
            cross: Justification::Start,
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
        self.attribute |= attribute;
        self
    }

    pub fn bold(self) -> Self {
        self.attribute(Attribute::BOLD)
    }

    pub fn faint(self) -> Self {
        self.attribute(Attribute::FAINT)
    }

    pub fn italic(self) -> Self {
        self.attribute(Attribute::ITALIC)
    }

    pub fn underline(self) -> Self {
        self.attribute(Attribute::UNDERLINE)
    }

    pub fn blink(self) -> Self {
        self.attribute(Attribute::BLINK)
    }

    pub fn reverse(self) -> Self {
        self.attribute(Attribute::REVERSE)
    }

    pub fn strikeout(self) -> Self {
        self.attribute(Attribute::STRIKEOUT)
    }

    pub fn main(mut self, main: Justification) -> Self {
        self.main = main;
        self
    }

    pub fn cross(mut self, cross: Justification) -> Self {
        self.cross = cross;
        self
    }

    pub fn size(&self) -> Vec2 {
        vec2(self.text.measure() as _, 1)
    }

    pub fn draw(&self, rect: Rect, surface: &mut Surface) {
        struct Line<'a> {
            grapheme: GraphemeKind<'a>,
            x: i32,
            y: i32,
        }

        let mut temp = String::new();
        let mut lines = Vec::new();

        let available_width = rect.width();
        let available_height = rect.height();

        let mut width = 0;

        for grapheme in self.text.graphemes() {
            let w = grapheme.width() as i32;
            if w + width > available_width {
                let x = match self.main {
                    Justification::Start => 0,
                    Justification::Center => (available_width - width) / 2,
                    Justification::End => available_width - width,
                };

                let x = x + rect.left();
                let y = lines.len() as i32;
                let grapheme = GraphemeKind::Cluster(Cow::from(std::mem::take(&mut temp)));
                lines.push(Line { grapheme, x, y });
                width = 0;
            }

            match grapheme {
                GraphemeKind::Cluster(c) => {
                    temp.push_str(&c);
                }
                GraphemeKind::Scalar(s) => temp.push(s),
            }
            width += w;
        }

        if !temp.is_empty() {
            let x = match self.main {
                Justification::Start => 0,
                Justification::Center => (available_width - width) / 2,
                Justification::End => available_width - width,
            };

            let x = x + rect.left();
            let grapheme = GraphemeKind::Cluster(Cow::from(&temp));
            let y = lines.len() as i32;
            lines.push(Line { grapheme, x, y });
        }

        let total = lines.len() as i32;

        let y = match self.cross {
            Justification::Start => rect.top(),
            Justification::Center => rect.top() + (available_height - total) / 2,
            Justification::End => rect.bottom() - total,
        };

        for line in lines {
            let cell = match line.grapheme {
                GraphemeKind::Cluster(c) => Cell::Grapheme(
                    Grapheme::new(c)
                        .fg(self.fg)
                        .bg(self.bg)
                        .attribute(self.attribute),
                ),
                GraphemeKind::Scalar(s) => Cell::Pixel(
                    Pixel::new(s)
                        .fg(self.fg)
                        .bg(self.bg)
                        .attribute(self.attribute),
                ),
            };
            surface.set(pos2(line.x, line.y + y), cell);
        }
    }
}

mod util {
    pub fn count_signed_digits(d: isize) -> usize {
        let signed = d.is_negative() as usize;
        let len = count_digits(d.unsigned_abs());
        len + signed
    }

    pub fn signed_digits(d: isize) -> impl Iterator<Item = char> {
        d.is_negative()
            .then_some('-')
            .into_iter()
            .chain(digits(d.unsigned_abs()))
    }

    pub fn count_digits(d: usize) -> usize {
        let (mut len, mut n) = (1, 1);
        while len < 20 {
            n *= 10;
            if n > d {
                return len;
            }
            len += 1;
        }
        len
    }

    pub fn digits(mut d: usize) -> impl Iterator<Item = char> {
        let x = count_digits(d) as u32 - 1;
        let mut mag = 10usize.pow(x);
        if d < mag {
            mag /= 10
        }

        let mut is_zero = d == 0;
        std::iter::from_fn(move || {
            if std::mem::take(&mut is_zero) {
                return Some(0);
            }
            if mag == 0 {
                return None;
            }
            let n = d / mag;
            d %= mag;
            mag /= 10;
            Some(n as u8)
        })
        .map(int_to_char)
    }

    pub const fn int_to_char(c: u8) -> char {
        (c + b'0') as char
    }
}
