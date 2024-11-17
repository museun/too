use crate::{
    layout::{Axis, Flex},
    math::{Size, Space},
    renderer::{Pixel, Rgba},
    view::{Builder, Elements, Layout, Palette, Render, StyleKind, View},
};

#[derive(Debug, Copy, Clone)]
#[must_use = "a view does nothing unless `show()` or `show_children()` is called"]
pub struct Expander;

impl<'v> Builder<'v> for Expander {
    type View = Self;
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

pub type SeparatorClass = fn(&Palette, Axis) -> SeparatorStyle;

#[derive(Debug, Copy, Clone)]
pub struct SeparatorStyle {
    pub fg: Rgba,
    pub bg: Option<Rgba>,
    pub pixel: char,
}

impl SeparatorStyle {
    pub fn double(palette: &Palette, axis: Axis) -> Self {
        Self {
            fg: palette.outline,
            bg: None,
            pixel: axis.cross((
                Elements::DOUBLE_HORIZONATAL_LINE,
                Elements::DOUBLE_VERTICAL_LINE,
            )),
        }
    }

    pub fn thick(palette: &Palette, axis: Axis) -> Self {
        Self {
            fg: palette.outline,
            bg: None,
            pixel: axis.cross((
                Elements::THICK_HORIZONTAL_LINE,
                Elements::THICK_VERTICAL_LINE,
            )),
        }
    }

    pub fn thin(palette: &Palette, axis: Axis) -> Self {
        Self {
            fg: palette.outline,
            bg: None,
            pixel: axis.cross((
                Elements::HORIZONTAL_LINE, //
                Elements::VERTICAL_LINE,
            )),
        }
    }

    pub fn thin_dashed(palette: &Palette, axis: Axis) -> Self {
        Self {
            fg: palette.outline,
            bg: None,
            pixel: axis.cross((
                Elements::DASH_HORIZONTAL_LINE, //
                Elements::DASH_VERTICAL_LINE,
            )),
        }
    }

    pub fn thick_dashed(palette: &Palette, axis: Axis) -> Self {
        Self {
            fg: palette.outline,
            bg: None,
            pixel: axis.cross((
                Elements::THICK_DASH_HORIZONTAL_LINE,
                Elements::THICK_DASH_VERTICAL_LINE,
            )),
        }
    }
}

#[derive(Debug, Copy, Clone)]
#[must_use = "a view does nothing unless `show()` or `show_children()` is called"]
pub struct Separator {
    class: StyleKind<SeparatorClass, SeparatorStyle>,
}

pub const fn separator() -> Separator {
    Separator {
        class: StyleKind::Deferred(SeparatorStyle::thick),
    }
}

impl Separator {
    pub const fn class(mut self, class: SeparatorClass) -> Self {
        self.class = StyleKind::Deferred(class);
        self
    }

    pub const fn style(mut self, style: SeparatorStyle) -> Self {
        self.class = StyleKind::Direct(style);
        self
    }
}

impl<'v> Builder<'v> for Separator {
    type View = Self;
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

        let style = match self.class {
            StyleKind::Deferred(style) => (style)(render.palette, axis),
            StyleKind::Direct(style) => style,
        };

        let mut pixel = Pixel::new(style.pixel).fg(style.fg);
        if let Some(bg) = style.bg {
            pixel = pixel.bg(bg)
        }

        render.fill_with(pixel);
    }
}
