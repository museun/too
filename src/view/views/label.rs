use compact_str::{CompactString, ToCompactString};
use unicode_segmentation::UnicodeSegmentation as _;
use unicode_width::UnicodeWidthStr as _;

use crate::{
    view::{
        geom::{Size, Space},
        Builder, Layout, Render, Styled, Theme, View,
    },
    Attribute, Grapheme, Rgba,
};

use super::measure_text;

pub fn label(label: impl ToCompactString) -> Label {
    Label {
        label: label.to_compact_string(),
        attr: None,
        fg: None,
    }
}

#[derive(Debug, Clone)]
#[must_use = "a view does nothing unless `show()` or `show_children()` is called"]
pub struct Label {
    label: CompactString,
    attr: Option<Attribute>,
    fg: Option<Rgba>,
}

impl Label {
    pub fn fg(mut self, fg: impl Into<Rgba>) -> Self {
        self.fg = Some(fg.into());
        self
    }

    pub fn italic(self) -> Self {
        self.attribute(Attribute::ITALIC)
    }

    pub fn bold(self) -> Self {
        self.attribute(Attribute::BOLD)
    }

    pub fn underline(self) -> Self {
        self.attribute(Attribute::UNDERLINE)
    }

    pub fn faint(self) -> Self {
        self.attribute(Attribute::FAINT)
    }

    pub fn blink(self) -> Self {
        self.attribute(Attribute::BLINK)
    }

    pub fn strikeout(self) -> Self {
        self.attribute(Attribute::STRIKEOUT)
    }

    pub fn attribute(mut self, attribute: Attribute) -> Self {
        match &mut self.attr {
            Some(old) => *old |= attribute,
            this @ None => *this = Some(attribute),
        }
        self
    }
}

impl<'v> Builder<'v> for Label {
    type View = Self;
}

impl View for Label {
    type Args<'v> = Self;
    type Response = ();

    fn create(args: Self::Args<'_>) -> Self {
        args
    }

    fn layout(&mut self, layout: Layout, space: Space) -> Size {
        // TODO support wrapping | truncation
        space.fit(measure_text(&self.label))
    }

    fn draw(&mut self, mut render: Render) {
        let mut start = 0;
        let fg = self.fg.unwrap_or(render.theme.foreground);
        for grapheme in self.label.graphemes(true) {
            let mut cell = Grapheme::new(grapheme).fg(fg);
            if let Some(attr) = self.attr {
                cell = cell.attribute(attr);
            }
            render.surface.set((start, 0), cell);
            start += grapheme.width() as i32
        }
    }
}
