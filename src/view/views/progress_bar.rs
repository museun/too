use std::ops::RangeInclusive;

use crate::{
    layout::Axis,
    math::{lerp, normalize, Vec2},
    view::{
        geom::{Size, Space},
        style::Theme,
        AxisProperty, Builder, Elements, Layout, Render, Styled, View,
    },
    Pixel, Rgba,
};

#[derive(Debug, Clone)]
#[must_use = "a view does nothing unless `show()` or `show_children()` is called"]
pub struct ProgressBar {
    value: f32,
    range: RangeInclusive<f32>,
    axis: Axis,
}

impl ProgressBar {
    pub const FILLED: Styled<AxisProperty<char>> = Styled::new(
        "too.progress_bar.filled",
        AxisProperty::same(Elements::MEDIUM_RECT),
    );

    pub const UNFILLED: Styled<AxisProperty<char>> = Styled::new(
        "too.progress_bar.unfilled",
        AxisProperty::new(
            Elements::THICK_DASH_HORIZONTAL_LINE,
            Elements::THICK_DASH_VERTICAL_LINE,
        ),
    );

    pub const FILLED_COLOR: Styled<Rgba> =
        Styled::new("too.progress_bar.filled.color", Theme::dark().primary);

    pub const UNFILLED_COLOR: Styled<Rgba> =
        Styled::new("too.progress_bar.unfilled.color", Theme::dark().surface);

    pub const SIZE: Styled<AxisProperty<f32>> = Styled::new(
        "too.progress_bar.size", //
        AxisProperty::new(20.0, 10.0),
    );
}

impl ProgressBar {
    pub const fn new(value: f32) -> Self {
        Self {
            value,
            range: 0.0..=1.0,
            axis: Axis::Horizontal,
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
}

impl<'v> Builder<'v> for ProgressBar {
    type View = Self;
}

impl View for ProgressBar {
    type Args<'v> = Self;
    type Response = ();

    fn create(args: Self::Args<'_>) -> Self {
        args
    }

    fn layout(&mut self, mut layout: Layout, space: Space) -> Size {
        let main = self.axis.main(layout.property(Self::SIZE));
        let size = self.axis.pack(main, 1.0);
        space.fit(size)
    }

    fn draw(&mut self, mut render: Render) {
        let rect = render.surface.rect();

        // TODO axis
        let axis = self.axis;

        let unfilled = render.property(Self::UNFILLED).resolve(axis);
        let bg = render.property(Self::UNFILLED_COLOR);

        render.surface.fill_with(Pixel::new(unfilled).fg(bg));

        let x = normalize(self.value, self.range.clone());

        let extent = axis.main(rect.size());
        let x = lerp(0.0, extent, x);

        let filled = render.property(Self::FILLED).resolve(axis);
        let bg = render.property(Self::FILLED_COLOR);

        let pos: Vec2 = axis.pack(x, axis.cross(rect.size()));
        let pixel = Pixel::new(filled).fg(bg);
        render.surface.fill_up_to_with(pos, pixel);
    }
}

pub const fn progress_bar(value: f32) -> ProgressBar {
    ProgressBar::new(value)
}
