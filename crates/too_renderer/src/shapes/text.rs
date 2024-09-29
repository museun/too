use crate::{Attribute, Color, Pixel, Shape};

use too_math::{
    layout::{Align, Align2},
    midpoint, pos2, rect, Pos2, Rect, Vec2,
};

use super::Label;

pub struct Text<T: Label> {
    fg: Color,
    bg: Color,
    attr: Option<Attribute>,
    align: Align2,
    pub label: T,
}

impl<T: Label> Text<T> {
    pub const fn new(label: T) -> Self {
        Self {
            fg: Color::Reset,
            bg: Color::Reuse,
            attr: None,
            align: Align2::LEFT_TOP,
            label,
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

    pub const fn attribute(mut self, attr: Attribute) -> Self {
        self.attr = Some(attr);
        self
    }

    pub const fn maybe_attribute(mut self, attr: Option<Attribute>) -> Self {
        self.attr = attr;
        self
    }

    pub const fn bold(self) -> Self {
        self.attribute(Attribute::BOLD)
    }

    pub const fn faint(self) -> Self {
        self.attribute(Attribute::FAINT)
    }

    pub const fn italic(self) -> Self {
        self.attribute(Attribute::ITALIC)
    }

    pub const fn underline(self) -> Self {
        self.attribute(Attribute::UNDERLINE)
    }

    pub const fn blink(self) -> Self {
        self.attribute(Attribute::BLINK)
    }

    pub const fn reverse(self) -> Self {
        self.attribute(Attribute::REVERSE)
    }

    pub const fn strikeout(self) -> Self {
        self.attribute(Attribute::STRIKEOUT)
    }

    pub const fn h_align(mut self, align: Align) -> Self {
        self.align.x = align;
        self
    }

    pub const fn v_align(mut self, align: Align) -> Self {
        self.align.y = align;
        self
    }

    pub const fn align2(mut self, align: Align2) -> Self {
        self.align = align;
        self
    }

    pub fn is_empty(&self) -> bool {
        self.size().x == 0
    }

    pub fn size(&self) -> Vec2 {
        self.label.size()
    }

    pub fn into_static(self) -> Text<T::Static> {
        Text {
            fg: self.fg,
            bg: self.bg,
            attr: self.attr,
            align: self.align,
            label: self.label.into_static(),
        }
    }

    // TODO crop and wrap
    pub fn render(&self, rect: Rect, mut put: impl FnMut(Pos2, Pixel)) {
        let origin = rect.left_top();
        let size = rect.size();

        let item_size = self.label.size();
        let x = match self.align.x {
            Align::Min => 0,
            Align::Center => midpoint(size.x, item_size.x),
            Align::Max => size.x.saturating_sub(item_size.x),
        };
        let y = match self.align.y {
            Align::Min => 0,
            Align::Center => midpoint(size.y, item_size.y), // BUG this is off by +1
            Align::Max => size.y.saturating_sub(item_size.y),
        };

        let mut start = pos2(x, y);
        // TODO better utf-8 handling
        for ch in self.label.chars() {
            if start.x >= size.x || start.y >= size.y {
                break;
            }

            if ch == '\n' {
                start.y += 1;
                start.x = x;
                continue;
            }

            if ch.is_ascii_control() {
                continue;
            }

            let pixel = Pixel::new(ch).fg(self.fg).bg(self.bg).attr(self.attr);
            put(start + origin, pixel);
            start.x += 1;
        }
    }
}

impl<T: Label> Shape for Text<T> {
    fn draw(&self, size: Vec2, put: impl FnMut(Pos2, Pixel)) {
        self.render(rect(size), put);
    }
}

impl<T: Label + Default> Default for Text<T> {
    fn default() -> Self {
        Self::new(T::default())
    }
}
