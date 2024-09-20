use too_math::{midpoint, pos2, Align, Align2, Pos2, Vec2};
use too_renderer::{Attribute, Color, Pixel, Shape};

use crate::Label;

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

    pub const fn attr(mut self, attr: Attribute) -> Self {
        self.attr = Some(attr);
        self
    }

    pub const fn bold(self) -> Self {
        self.attr(Attribute::BOLD)
    }

    pub const fn faint(self) -> Self {
        self.attr(Attribute::FAINT)
    }

    pub const fn italic(self) -> Self {
        self.attr(Attribute::ITALIC)
    }

    pub const fn underline(self) -> Self {
        self.attr(Attribute::UNDERLINE)
    }

    pub const fn blink(self) -> Self {
        self.attr(Attribute::BLINK)
    }

    pub const fn reverse(self) -> Self {
        self.attr(Attribute::REVERSE)
    }

    pub const fn strikeout(self) -> Self {
        self.attr(Attribute::STRIKEOUT)
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
}

impl<T: Label> Shape for Text<T> {
    fn draw(&self, size: Vec2, mut put: impl FnMut(Pos2, Pixel)) {
        let item_size = self.label.size();
        let x = match self.align.x {
            Align::Min => 0,
            Align::Center => midpoint(size.x, item_size.x),
            Align::Max => size.x.saturating_sub(item_size.x),
        };
        let y = match self.align.y {
            Align::Min => 0,
            Align::Center => midpoint(size.y, item_size.y),
            Align::Max => size.y.saturating_sub(item_size.y),
        };

        // TODO crop and wrap
        let mut start = pos2(x, y);
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
            put(start, pixel);
            start.x += 1;
        }
    }
}

impl<T: Label + Default> Default for Text<T> {
    fn default() -> Self {
        Self::new(T::default())
    }
}
