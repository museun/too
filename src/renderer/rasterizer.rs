use std::ops::RangeInclusive;

use crate::{
    layout::Axis,
    math::{pos2, Pos2, Rect},
    renderer::{Attribute, Cell, Color, Grapheme, Pixel, Rgba},
    view::ViewId,
    Str,
};

/// Shapes that a [`Rasterizer`] can produce
#[derive(Clone, PartialEq, Eq)]
pub enum Shape {
    /// Fill the region with a color
    FillBg {
        /// The region to fill
        rect: Rect,
        /// The color to use
        color: Rgba,
    },
    /// Fill the region with a pixel
    FillWith {
        /// The region to fill
        rect: Rect,
        /// The [`Pixel`] to use
        pixel: Pixel,
    },
    /// Draw a line between 2 positions, wtih a pixel
    Line {
        /// The start position
        start: Pos2,
        /// The end position
        end: Pos2,
        /// The pixel to use
        pixel: Pixel,
    },
    /// Draws some text into a region
    Text {
        /// The region to use
        rect: Rect,
        /// The Text to use
        shape: TextShape,
    },
    /// Set a specific cell at a position with a [`Cell`]
    Set {
        /// The position
        pos: Pos2,
        /// The cell to use
        cell: Cell,
    },
}

impl std::fmt::Debug for Shape {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        struct CompactRect<'a>(&'a Rect);
        impl std::fmt::Debug for CompactRect<'_> {
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

/// A rasterizer turns abstract shapes into _draw_ calls
pub trait Rasterizer {
    /// Start a new shape with an provided id
    fn begin(&mut self, _id: ViewId) {}
    /// End the current shape with the provided id
    fn end(&mut self, _id: ViewId) {}

    /// Sets the region for this rasterizer to use
    fn set_rect(&mut self, rect: Rect);
    /// The region that this rasterizer is using
    fn rect(&self) -> Rect;

    /// Clear the entire region with a color
    fn clear(&mut self, color: Rgba) {
        self.fill_bg(color);
    }

    /// Patch the background region
    fn patch(&mut self, rect: Rect, patch: &dyn Fn(&mut Cell));

    /// Fill the background of the region with a color
    fn fill_bg(&mut self, color: Rgba);
    /// Fill the background of the region with a pixel
    fn fill_with(&mut self, pixel: Pixel);

    /// Draw a horizontal line at the `y` offset between `x0..=x1` using the provided pixel
    fn horizontal_line(&mut self, y: i32, range: RangeInclusive<i32>, pixel: Pixel) {
        self.line(Axis::Horizontal, pos2(0, y), range, pixel)
    }
    /// Draw a vertical line at the `x` offset between `y0..=y1` using the provided pixel
    fn vertical_line(&mut self, x: i32, range: RangeInclusive<i32>, pixel: Pixel) {
        self.line(Axis::Vertical, pos2(x, 0), range, pixel)
    }

    /// Draws a line in a specific orientation starting an offset `x0,x1..=y0,y1` using the provided pixel
    fn line(&mut self, axis: Axis, offset: Pos2, range: RangeInclusive<i32>, pixel: Pixel);

    /// Draws a [`TextShape`] into the region
    fn text(&mut self, shape: TextShape);

    /// Sets a pixel as a specific position
    fn pixel(&mut self, pos: Pos2, pixel: Pixel);
    /// Sets a grapheme as a specific position
    fn grapheme(&mut self, pos: Pos2, grapheme: Grapheme);
    /// Tries to get a cell at a specific position
    fn get_mut(&mut self, pos: Pos2) -> Option<&mut Cell>;
}

/// A shape for drawing text
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TextShape {
    pub(crate) label: Str,
    pub(crate) fg: Color,
    pub(crate) bg: Color,
    pub(crate) attribute: Option<Attribute>,
}

impl From<&str> for TextShape {
    fn from(value: &str) -> Self {
        Self::new(value)
    }
}

impl From<Str> for TextShape {
    fn from(value: Str) -> Self {
        Self::new(value)
    }
}

impl TextShape {
    /// Create a new text shape from a label.
    ///
    /// By default, when drawn ontop of another shape, this will:
    /// - reset the foreground
    /// - reuse the background
    pub fn new(label: impl Into<Str>) -> Self {
        Self {
            label: label.into(),
            fg: Color::Reset,
            bg: Color::Reuse,
            attribute: None,
        }
    }

    pub const fn const_new(label: Str) -> Self {
        Self {
            label,
            fg: Color::Reset,
            bg: Color::Reuse,
            attribute: None,
        }
    }

    /// Reuse the existing foreground
    ///
    /// By default, the foreground is reset. This changes that.
    pub fn reuse_fg(mut self) -> Self {
        self.fg = Color::Reuse;
        self
    }

    /// Reset the existing background
    ///
    /// By default, the background is reused. This changes that.
    pub fn reset_bg(mut self) -> Self {
        self.bg = Color::Reset;
        self
    }

    /// Sets the foreground to use for this label
    pub fn fg(mut self, fg: impl Into<Rgba>) -> Self {
        self.fg = Color::Set(fg.into());
        self
    }

    /// Sets the background to use for this label
    pub fn bg(mut self, bg: impl Into<Rgba>) -> Self {
        self.bg = Color::Set(bg.into());
        self
    }

    /// Use this attribute for the label
    pub fn attribute(mut self, attribute: Attribute) -> Self {
        match &mut self.attribute {
            Some(attr) => *attr |= attribute,
            None => self.attribute = Some(attribute),
        }
        self
    }

    /// Use this attribute for the label, maybe
    pub fn maybe_attribute(self, attribute: Option<Attribute>) -> Self {
        match attribute {
            Some(attr) => self.attribute(attr),
            None => self,
        }
    }
}
