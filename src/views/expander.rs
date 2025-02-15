use crate::{
    layout::{Axis, Flex},
    math::{Size, Space},
    renderer::{Pixel, Rgba},
    view::{
        ApplicableStyle, Builder, Elements, Layout, Palette, Render, Style, StyleOptions, View,
    },
};

#[derive(Debug, Copy, Clone)]
#[must_use = "a view does nothing unless `show()` or `show_children()` is called"]
pub struct Expander;

impl Builder<'_> for Expander {
    type View = Self;
    type Style = ();
}

impl View for Expander {
    type Args<'v> = Self;
    type Response = ();

    fn create(args: Self::Args<'_>) -> Self {
        args
    }

    fn flex(&self) -> Flex {
        Flex::Tight(1.0)
    }

    fn layout(&mut self, layout: Layout, space: Space) -> Size {
        let axis = layout.parent_axis();
        axis.pack(axis.main(space.max.finite_or_zero()), 0.0)
    }
}

pub const fn expander() -> Expander {
    Expander
}

#[derive(Debug, Copy, Clone)]
pub struct SeparatorStyle {
    pub fg: Rgba,
    pub bg: Option<Rgba>,
    pub pixel: char,
}

impl Style for SeparatorStyle {
    type Args = Axis;
    fn default(palette: &Palette, options: StyleOptions<Axis>) -> Self {
        Self::thick(palette, options)
    }
}

impl SeparatorStyle {
    pub fn double(palette: &Palette, options: StyleOptions<Axis>) -> Self {
        Self {
            fg: palette.outline,
            bg: None,
            pixel: options.cross((
                Elements::DOUBLE_HORIZONATAL_LINE,
                Elements::DOUBLE_VERTICAL_LINE,
            )),
        }
    }

    pub fn thick(palette: &Palette, options: StyleOptions<Axis>) -> Self {
        Self {
            fg: palette.outline,
            bg: None,
            pixel: options.cross((
                Elements::THICK_HORIZONTAL_LINE,
                Elements::THICK_VERTICAL_LINE,
            )),
        }
    }

    pub fn thin(palette: &Palette, options: StyleOptions<Axis>) -> Self {
        Self {
            fg: palette.outline,
            bg: None,
            pixel: options.cross((
                Elements::HORIZONTAL_LINE, //
                Elements::VERTICAL_LINE,
            )),
        }
    }

    pub fn thin_dashed(palette: &Palette, options: StyleOptions<Axis>) -> Self {
        Self {
            fg: palette.outline,
            bg: None,
            pixel: options.cross((
                Elements::DASH_HORIZONTAL_LINE, //
                Elements::DASH_VERTICAL_LINE,
            )),
        }
    }

    pub fn thick_dashed(palette: &Palette, options: StyleOptions<Axis>) -> Self {
        Self {
            fg: palette.outline,
            bg: None,
            pixel: options.cross((
                Elements::THICK_DASH_HORIZONTAL_LINE,
                Elements::THICK_DASH_VERTICAL_LINE,
            )),
        }
    }
}

#[derive(Debug)]
#[must_use = "a view does nothing unless `show()` or `show_children()` is called"]
pub struct Separator {
    style: ApplicableStyle<SeparatorStyle>,
}

pub fn separator() -> Separator {
    Separator {
        style: ApplicableStyle::default(),
    }
}

impl Builder<'_> for Separator {
    type View = Self;
    type Style = SeparatorStyle;

    fn applicable_style(&mut self) -> Option<&mut ApplicableStyle<Self::Style>> {
        Some(&mut self.style)
    }
}

impl View for Separator {
    type Args<'v> = Self;
    type Response = ();

    fn create(args: Self::Args<'_>) -> Self {
        args
    }

    fn flex(&self) -> Flex {
        Flex::Loose(1.0)
    }

    fn layout(&mut self, layout: Layout, space: Space) -> Size {
        let axis = layout.parent_axis();
        let main = axis.cross(space.max.finite_or_zero());
        space.constrain_min(axis.pack(1.0, main))
    }

    fn draw(&mut self, mut render: Render) {
        let axis = render.parent_axis();
        let style = self.style.apply(&render, |s| s.with_args(axis));

        let mut pixel = Pixel::new(style.pixel).fg(style.fg);
        if let Some(bg) = style.bg {
            pixel = pixel.bg(bg)
        }

        render.fill_with(pixel);
    }
}
