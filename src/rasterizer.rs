use std::{borrow::Cow, ops::RangeInclusive};

use crate::{
    layout::Axis,
    math::{pos2, Pos2, Rect},
    view::ViewId,
    Attribute, Cell, Color, Grapheme, Pixel, Rgba,
};

#[derive(Clone, PartialEq)]
pub enum Shape {
    FillBg {
        rect: Rect,
        color: Rgba,
    },
    FillWith {
        rect: Rect,
        pixel: Pixel,
    },
    Line {
        start: Pos2,
        end: Pos2,
        pixel: Pixel,
    },
    Text {
        rect: Rect,
        shape: TextShape<'static>,
    },
    Set {
        pos: Pos2,
        cell: Cell,
    },
}

impl std::fmt::Debug for Shape {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        struct CompactRect<'a>(&'a Rect);
        impl<'a> std::fmt::Debug for CompactRect<'a> {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                write!(
                    f,
                    "{{ x: {}, y: {}, w: {}, h: {} }}",
                    self.0.min.x,
                    self.0.min.y,
                    self.0.width(),
                    self.0.height()
                )
            }
        }

        match self {
            Self::FillBg { rect, color } => f
                .debug_struct("FillBg")
                .field("rect", &CompactRect(rect))
                .field("color", color)
                .finish(),
            Self::FillWith { rect, pixel } => f
                .debug_struct("FillWith")
                .field("rect", &CompactRect(rect))
                .field("pixel", pixel)
                .finish(),
            Self::Line { start, end, pixel } => f
                .debug_struct("Line")
                .field("start", start)
                .field("end", end)
                .field("pixel", pixel)
                .finish(),
            Self::Text { rect, shape } => f
                .debug_struct("Text")
                .field("rect", &CompactRect(rect))
                .field("shape", shape)
                .finish(),
            Self::Set { pos, cell } => f
                .debug_struct("Set")
                .field("pos", pos)
                .field("cell", cell)
                .finish(),
        }
    }
}

pub trait Rasterizer {
    fn begin(&mut self, _id: ViewId) {}
    fn end(&mut self, _id: ViewId) {}

    fn set_rect(&mut self, rect: Rect);
    fn rect(&self) -> Rect;

    fn clear(&mut self, color: Rgba) {
        self.fill_bg(color);
    }

    fn fill_bg(&mut self, color: Rgba);
    fn fill_with(&mut self, pixel: Pixel);
    fn horizontal_line(&mut self, y: i32, range: RangeInclusive<i32>, pixel: Pixel) {
        self.line(Axis::Horizontal, pos2(0, y), range, pixel)
    }
    fn vertical_line(&mut self, x: i32, range: RangeInclusive<i32>, pixel: Pixel) {
        self.line(Axis::Vertical, pos2(x, 0), range, pixel)
    }
    fn line(&mut self, axis: Axis, offset: Pos2, range: RangeInclusive<i32>, pixel: Pixel);
    fn text(&mut self, shape: TextShape<'_>);
    fn pixel(&mut self, pos: Pos2, pixel: Pixel);
    fn grapheme(&mut self, pos: Pos2, grapheme: Grapheme);
    fn get_mut(&mut self, pos: Pos2) -> Option<&mut Cell>;
}

#[derive(Clone, Debug, PartialEq)]
pub struct TextShape<'a> {
    pub(crate) label: Cow<'a, str>,
    pub(crate) fg: Color,
    pub(crate) bg: Color,
    pub(crate) attribute: Option<Attribute>,
}

impl<'a> From<&'a str> for TextShape<'a> {
    fn from(value: &'a str) -> Self {
        Self::new(value)
    }
}

impl<'a> TextShape<'a> {
    pub const fn new(label: &'a str) -> Self {
        Self {
            label: Cow::Borrowed(label),
            fg: Color::Reset,
            bg: Color::Reuse,
            attribute: None,
        }
    }

    pub fn fg(mut self, fg: impl Into<Rgba>) -> Self {
        self.fg = Color::Set(fg.into());
        self
    }

    pub fn bg(mut self, bg: impl Into<Rgba>) -> Self {
        self.bg = Color::Set(bg.into());
        self
    }

    pub fn attribute(mut self, attribute: Attribute) -> Self {
        match &mut self.attribute {
            Some(attr) => *attr |= attribute,
            None => self.attribute = Some(attribute),
        }
        self
    }

    pub fn maybe_attribute(mut self, attribute: Option<Attribute>) -> Self {
        match attribute {
            Some(attr) => self.attribute(attr),
            None => {
                self.attribute.take();
                self
            }
        }
    }
}
