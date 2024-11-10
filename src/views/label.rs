use compact_str::{CompactString, ToCompactString};
use unicode_segmentation::UnicodeSegmentation;
use unicode_width::UnicodeWidthStr;

use crate::{
    layout::Align,
    math::{pos2, Size, Space},
    view::{Builder, Layout, Palette, Render, StyleKind, View},
    Attribute, Grapheme, Rgba,
};

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct LabelStyle {
    pub foreground: Rgba,
}

impl LabelStyle {
    pub const fn default(palette: &Palette) -> LabelStyle {
        LabelStyle {
            foreground: palette.foreground,
        }
    }

    pub const fn info(palette: &Palette) -> LabelStyle {
        LabelStyle {
            foreground: palette.info,
        }
    }

    pub const fn warning(palette: &Palette) -> LabelStyle {
        LabelStyle {
            foreground: palette.warning,
        }
    }

    pub const fn danger(palette: &Palette) -> LabelStyle {
        LabelStyle {
            foreground: palette.danger,
        }
    }
}

pub type LabelClass = fn(&Palette) -> LabelStyle;

pub fn label(label: impl ToCompactString) -> Label {
    Label::new(label)
}

impl Label {
    pub fn new(label: impl ToCompactString) -> Self {
        Label {
            label: label.to_compact_string(),
            class: StyleKind::Deferred(LabelStyle::default),
            main: Align::Min,
            attribute: None,
        }
    }

    pub const fn class(mut self, class: LabelClass) -> Self {
        self.class = StyleKind::Deferred(class);
        self
    }

    pub const fn style(mut self, style: LabelStyle) -> Self {
        self.class = StyleKind::Direct(style);
        self
    }

    pub const fn horizontal_align(mut self, justify: Align) -> Self {
        self.main = justify;
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
        match &mut self.attribute {
            Some(old) => *old |= attribute,
            this @ None => *this = Some(attribute),
        }
        self
    }
}

#[must_use = "a view does nothing unless `show()` or `show_children()` is called"]
#[derive(Debug)]
pub struct Label {
    label: CompactString,
    class: StyleKind<LabelClass, LabelStyle>,
    main: Align,
    attribute: Option<Attribute>,
}

#[derive(Copy, Clone, Debug)]
pub struct Measure {
    pub truncate: Option<usize>,
    pub size: Size,
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
        space.fit(Size::new(self.label.width() as f32, 1.0))
    }

    fn draw(&mut self, mut render: Render) {
        let style = match self.class {
            StyleKind::Deferred(class) => (class)(render.palette),
            StyleKind::Direct(style) => style,
        };

        let local = render.local_rect();

        for (x, g) in self.label.graphemes(true).enumerate() {
            let mut cell = Grapheme::new(g).fg(style.foreground);
            if let Some(attr) = self.attribute {
                cell = cell.attribute(attr)
            }
            render.surface.set(pos2(x as i32, 0), cell);
        }
    }
}