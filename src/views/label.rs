use compact_str::CompactString;
use unicode_width::UnicodeWidthStr as _;

use crate::{
    layout::Align,
    math::{Size, Space},
    renderer::{Attribute, Rgba, TextShape},
    view::{ApplicableStyle, Builder, Layout, Palette, Render, Style, StyleOptions, View, ViewExt},
    Str,
};

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct LabelStyle {
    pub foreground: Rgba,
}

impl Style for LabelStyle {
    type Args = ();
    fn default(palette: &Palette, options: StyleOptions) -> Self {
        Self {
            foreground: { options.resolve_color(palette, |palette| palette.foreground) },
        }
    }
}

impl LabelStyle {
    pub fn info(palette: &Palette, options: StyleOptions) -> Self {
        Self {
            foreground: { options.resolve_color(palette, |palette| palette.info) },
        }
    }

    pub fn warning(palette: &Palette, options: StyleOptions) -> Self {
        Self {
            foreground: { options.resolve_color(palette, |palette| palette.warning) },
        }
    }

    pub fn success(palette: &Palette, options: StyleOptions) -> Self {
        Self {
            foreground: { options.resolve_color(palette, |palette| palette.success) },
        }
    }

    pub fn danger(palette: &Palette, options: StyleOptions) -> Self {
        Self {
            foreground: { options.resolve_color(palette, |palette| palette.danger) },
        }
    }
}

pub fn label(label: impl Into<Str>) -> Label {
    Label::new(label)
}

impl Label {
    pub fn new(label: impl Into<Str>) -> Self {
        Self {
            label: label.into().into_inner(),
            style: ApplicableStyle::default(),
            main: Align::Min,
            attribute: None,
        }
    }

    pub const fn horizontal_align(mut self, justify: Align) -> Self {
        self.main = justify;
        self
    }

    pub fn fg(self, fg: impl Into<Rgba>) -> Self {
        let foreground = fg.into();
        self.style(LabelStyle { foreground })
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
    style: ApplicableStyle<LabelStyle>,
    main: Align,
    attribute: Option<Attribute>,
}

impl Builder<'_> for Label {
    type View = Self;
    type Style = LabelStyle;

    fn applicable_style(&mut self) -> Option<&mut ApplicableStyle<Self::Style>> {
        Some(&mut self.style)
    }
}

impl View for Label {
    type Args<'v> = Self;
    type Response = ();

    fn create(args: Self::Args<'_>) -> Self {
        args
    }

    fn layout(&mut self, _layout: Layout, space: Space) -> Size {
        space.fit(Size::new(self.label.width() as f32, 1.0))
    }

    fn draw(&mut self, mut render: Render) {
        let style = self.style.apply(&render, std::convert::identity);

        render.text(
            TextShape::new(&self.label)
                .fg(style.foreground)
                .maybe_attribute(self.attribute),
        );
    }
}
