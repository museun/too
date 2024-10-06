use std::ops::RangeInclusive;

use crate::{
    layout::Axis,
    math::{lerp, normalize},
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

    pub const WIDTH: Styled<f32> = Styled::new("too.progress_bar.width", 20.0);
    pub const HEIGHT: Styled<f32> = Styled::new("too.progress_bar.height", 1.0);
}

impl ProgressBar {
    pub const fn new(value: f32) -> Self {
        Self {
            value,
            range: 0.0..=1.0,
        }
    }

    pub const fn range(mut self, range: RangeInclusive<f32>) -> Self {
        self.range = range;
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

    fn layout(&mut self, layout: Layout, space: Space) -> Size {
        // TODO axis
        let w = layout.stylesheet.get_or_default(Self::WIDTH);
        let h = layout.stylesheet.get_or_default(Self::HEIGHT);
        space.fit(Size::new(w, h))
    }

    fn draw(&mut self, mut render: Render) {
        let rect = render.surface.rect();

        // TODO axis
        let axis = Axis::Horizontal;

        let unfilled = render
            .stylesheet
            .get_or_default(Self::UNFILLED)
            .resolve(axis);
        let bg = render.stylesheet.get_or_default(Self::UNFILLED_COLOR);

        render.surface.fill_with(Pixel::new(unfilled).fg(bg));
        let x = normalize(self.value, self.range.clone());
        let x = lerp(0.0, rect.width() as f32, x);

        let filled = render.stylesheet.get_or_default(Self::FILLED).resolve(axis);
        let bg = render.stylesheet.get_or_default(Self::FILLED_COLOR);

        render.surface.fill_up_to_with(
            (x, render.surface.rect.height() as f32),
            Pixel::new(filled).fg(bg),
        );
    }
}

pub const fn progress_bar(value: f32) -> ProgressBar {
    ProgressBar::new(value)
}
