use std::ops::RangeInclusive;

use crate::{
    layout::Axis,
    math::{lerp, normalize, Pos2},
    view::{
        geom::{Size, Space},
        style::StyleKind,
        Builder, Elements, Layout, Palette, Render, View,
    },
    Pixel, Rgba,
};

pub type ProgressClass = fn(&Palette, Axis) -> ProgressStyle;

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct ProgressStyle {
    pub unfilled_color: Rgba,
    pub filled_color: Rgba,
    pub unfilled_hovered: Option<Rgba>,
    pub filled_hovered: Option<Rgba>,
    pub unfilled: char,
    pub filled: char,
}

impl ProgressStyle {
    pub fn default(palette: &Palette, axis: Axis) -> Self {
        Self {
            unfilled_color: palette.outline,
            filled_color: palette.primary,
            unfilled_hovered: None,
            filled_hovered: None,
            unfilled: axis.main((Elements::MEDIUM_RECT, Elements::LARGE_RECT)),
            filled: Elements::LARGE_RECT,
        }
    }

    pub fn medium_filled(palette: &Palette, axis: Axis) -> Self {
        Self {
            unfilled: axis.main((Elements::MEDIUM_RECT, Elements::MEDIUM_RECT)),
            filled: axis.main((Elements::MEDIUM_RECT, Elements::MEDIUM_RECT)),
            ..Self::default(palette, axis)
        }
    }

    pub fn filled(palette: &Palette, axis: Axis) -> Self {
        Self {
            unfilled: axis.main((Elements::LARGE_RECT, Elements::LARGE_RECT)),
            filled: axis.main((Elements::LARGE_RECT, Elements::LARGE_RECT)),
            ..Self::default(palette, axis)
        }
    }

    pub fn thin(palette: &Palette, axis: Axis) -> Self {
        Self {
            unfilled: axis.main((Elements::HORIZONTAL_LINE, Elements::VERTICAL_LINE)),
            filled: axis.main((Elements::HORIZONTAL_LINE, Elements::VERTICAL_LINE)),
            ..Self::default(palette, axis)
        }
    }

    pub fn thick(palette: &Palette, axis: Axis) -> Self {
        Self {
            unfilled: axis.main((
                Elements::THICK_HORIZONTAL_LINE,
                Elements::THICK_VERTICAL_LINE,
            )),
            filled: axis.main((
                Elements::THICK_HORIZONTAL_LINE,
                Elements::THICK_VERTICAL_LINE,
            )),
            ..Self::default(palette, axis)
        }
    }

    pub fn thin_dashed(palette: &Palette, axis: Axis) -> Self {
        Self {
            unfilled: axis.main((Elements::DASH_HORIZONTAL_LINE, Elements::DASH_VERTICAL_LINE)),
            filled: axis.main((Elements::DASH_HORIZONTAL_LINE, Elements::DASH_VERTICAL_LINE)),
            ..Self::default(palette, axis)
        }
    }

    pub fn thick_dashed(palette: &Palette, axis: Axis) -> ProgressStyle {
        ProgressStyle {
            unfilled: axis.main((
                Elements::THICK_DASH_HORIZONTAL_LINE,
                Elements::THICK_DASH_VERTICAL_LINE,
            )),
            filled: axis.main((
                Elements::THICK_DASH_HORIZONTAL_LINE,
                Elements::THICK_DASH_VERTICAL_LINE,
            )),
            ..Self::default(palette, axis)
        }
    }
}

#[derive(Debug)]
#[must_use = "a view does nothing unless `show()` or `show_children()` is called"]
pub struct Progress {
    value: f32,
    range: RangeInclusive<f32>,
    axis: Axis,
    class: StyleKind<ProgressClass, ProgressStyle>,
}

impl Progress {
    pub const fn new(value: f32) -> Self {
        Self {
            value,
            range: 0.0..=1.0,
            axis: Axis::Horizontal,
            class: StyleKind::deferred(ProgressStyle::default),
        }
    }

    pub const fn range(mut self, range: RangeInclusive<f32>) -> Self {
        self.range = range;
        self
    }

    pub const fn horizontal(self) -> Self {
        self.axis(Axis::Horizontal)
    }

    pub const fn vertical(self) -> Self {
        self.axis(Axis::Vertical)
    }

    pub const fn axis(mut self, axis: Axis) -> Self {
        self.axis = axis;
        self
    }

    pub const fn class(mut self, class: ProgressClass) -> Self {
        self.class = StyleKind::Deferred(class);
        self
    }

    pub const fn style(mut self, style: ProgressStyle) -> Self {
        self.class = StyleKind::Direct(style);
        self
    }
}

impl<'v> Builder<'v> for Progress {
    type View = Self;
}

impl View for Progress {
    type Args<'v> = Self;
    type Response = ();

    fn create(args: Self::Args<'_>) -> Self {
        args
    }

    fn layout(&mut self, layout: Layout, space: Space) -> Size {
        let main = self.axis.main((20.0, 10.0));
        let size = self.axis.pack(main, 1.0);
        space.fit(size)
    }

    fn draw(&mut self, mut render: Render) {
        let rect = render.rect();
        let axis = self.axis;

        let style = match self.class {
            StyleKind::Deferred(style) => (style)(render.palette, self.axis),
            StyleKind::Direct(style) => style,
        };

        let color = if render.is_hovered() {
            style.unfilled_hovered.unwrap_or(style.unfilled_color)
        } else {
            style.unfilled_color
        };
        render
            .surface
            .fill_with(Pixel::new(style.unfilled).fg(color));

        let x = normalize(self.value, self.range.clone());

        let extent = axis.main::<f32>(rect.size());
        let x = lerp(0.0, extent, x);
        let x = x.round() as f32;

        let color = if render.is_hovered() {
            style.filled_hovered.unwrap_or(style.filled_color)
        } else {
            style.filled_color
        };

        let cross = axis.cross(rect.size() - 1);
        let pixel = Pixel::new(style.filled).fg(color);
        for x in 0..x as i32 {
            let pos: Pos2 = axis.pack(x, cross);
            render.surface.set(pos, pixel);
        }
    }
}

pub const fn progress(value: f32) -> Progress {
    Progress::new(value)
}
