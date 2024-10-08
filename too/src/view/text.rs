use crate::{math::Rect, Attribute, Color, Grapheme, Rgba, Surface};
use unicode_width::UnicodeWidthStr as _;

use super::geom::{Rectf, Size};

pub(crate) struct Text<'a> {
    pub data: &'a str,
    pub fg: Rgba,
    pub bg: Color,
    pub attribute: Attribute,
}

impl<'a> Text<'a> {
    pub fn measure(data: &str) -> Size {
        let width = data.width();
        Size::new(width as f32, 1.0)
    }

    pub fn draw(&self, rect: Rectf, surface: &mut Surface) {
        surface.set(
            Rect::from(rect).left_top(),
            Grapheme::new(self.data)
                .fg(self.fg)
                .bg(self.bg)
                .attribute(self.attribute),
        );
    }
}
