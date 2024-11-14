use std::{borrow::Cow, ops::RangeInclusive};

use unicode_segmentation::UnicodeSegmentation;

use crate::{
    layout::Axis,
    math::{pos2, Pos2, Rect},
    view::{CroppedSurface, ViewId},
    Attribute, Cell, Color, Grapheme, Pixel, Rgba,
};

impl<'a> Rasterizer for CroppedSurface<'a> {
    fn set_rect(&mut self, rect: Rect) {
        self.clip_rect = rect;
    }

    fn rect(&self) -> Rect {
        self.clip_rect
    }

    fn fill_bg(&mut self, color: Rgba) {
        self.fill(color);
    }

    fn fill_with(&mut self, pixel: Pixel) {
        self.fill_with(pixel);
    }

    fn line(&mut self, axis: Axis, offset: Pos2, range: RangeInclusive<i32>, pixel: Pixel) {
        let cross: i32 = axis.cross(offset);

        let start: Pos2 = axis.pack(*range.start(), cross);
        let end: Pos2 = axis.pack(*range.end(), cross);

        for y in start.y..=end.y {
            for x in start.x..=end.x {
                self.set(pos2(x, y), pixel);
            }
        }
    }

    fn text(&mut self, shape: TextShape<'_>) {
        for (x, g) in shape.label.graphemes(true).enumerate() {
            let mut cell = Grapheme::new(g).fg(shape.fg);
            if let Some(attr) = shape.attribute {
                cell = cell.attribute(attr)
            }
            self.set(pos2(x as i32, 0), cell);
        }
    }

    fn pixel(&mut self, pos: Pos2, pixel: Pixel) {
        self.set(pos, pixel);
    }

    fn grapheme(&mut self, pos: Pos2, grapheme: Grapheme) {
        self.set(pos, grapheme);
    }

    fn get_mut(&mut self, pos: Pos2) -> Option<&mut Cell> {
        self.get_mut(pos)
    }
}

pub trait Rasterizer {
    fn begin(&mut self, id: ViewId) {}
    fn end(&mut self, id: ViewId) {}

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
            fg: Color::Reuse,
            bg: Color::Reset,
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
