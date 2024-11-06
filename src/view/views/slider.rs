use core::f32;
use std::ops::RangeInclusive;

use crate::{
    layout::Axis,
    math::{denormalize, inverse_lerp, lerp, normalize, Pos2},
    view::{
        geom::{Size, Space},
        style::{AxisProperty, Styled, Theme},
        Builder, Elements, EventCtx, Handled, Interest, Knob, Layout, Render, Ui, View, ViewEvent,
    },
    Pixel, Rgba,
};

pub fn slider(value: &mut f32) -> Slider {
    Slider {
        value,
        range: 0.0..=1.0,
        clickable: true,
        axis: Axis::Horizontal,
    }
}

// TODO step by
#[must_use = "a view does nothing unless `show()` or `show_children()` is called"]
pub struct Slider<'v> {
    value: &'v mut f32,
    range: RangeInclusive<f32>,
    clickable: bool,
    axis: Axis,
}

impl<'v> Slider<'v> {
    pub const fn range(mut self, range: RangeInclusive<f32>) -> Self {
        self.range = range;
        self
    }

    pub const fn clickable(mut self, clickable: bool) -> Self {
        self.clickable = clickable;
        self
    }

    pub const fn axis(mut self, axis: Axis) -> Self {
        self.axis = axis;
        self
    }

    pub const fn horizontal(self) -> Self {
        self.axis(Axis::Horizontal)
    }

    pub const fn vertical(self) -> Self {
        self.axis(Axis::Vertical)
    }
}

impl<'v> Slider<'v> {
    pub const TRACK: Styled<AxisProperty<char>> = Styled::new(
        "too.slider.track",
        AxisProperty::new(
            Elements::THICK_HORIZONTAL_LINE,
            Elements::THICK_VERTICAL_LINE,
        ),
    );
    pub const TRACK_COLOR: Styled<Rgba> = Styled::new(
        "too.slider.track.color", //
        Theme::SURFACE.default(),
    );

    pub const KNOB: Styled<char> = Styled::new(
        "too.slider.knob", //
        Knob::ROUND,
    );
    pub const KNOB_COLOR: Styled<Rgba> = Styled::new(
        "too.slider.knob.color", //
        Theme::PRIMARY.default(),
    );

    pub const SIZE: Styled<AxisProperty<f32>> = Styled::new(
        "too.slider.size", //
        AxisProperty::new(20.0, 10.0),
    );
}

impl<'v> Builder<'v> for Slider<'v> {
    type View = SliderView;
}

#[derive(Debug)]
pub struct SliderView {
    value: f32,
    changed: bool,
    range: RangeInclusive<f32>,
    clickable: bool,
    axis: Axis,
}

impl View for SliderView {
    type Args<'v> = Slider<'v>;
    type Response = (); // TODO `changed`

    fn create(args: Self::Args<'_>) -> Self {
        Self {
            changed: false,
            value: *args.value,
            range: args.range.clone(),
            clickable: args.clickable,
            axis: args.axis,
        }
    }

    fn update(&mut self, args: Self::Args<'_>, ui: &Ui) -> Self::Response {
        self.range = args.range.clone();
        self.clickable = args.clickable;
        self.axis = args.axis;

        if std::mem::take(&mut self.changed) {
            *args.value = self.value;
        } else if self.value != *args.value {
            self.value = *args.value;
        }
    }

    fn interests(&self) -> Interest {
        Interest::MOUSE
    }

    fn event(&mut self, event: ViewEvent, ctx: EventCtx) -> Handled {
        let pos = match event {
            ViewEvent::MouseDrag {
                current,
                inside: true,
                ..
            } => current,

            ViewEvent::MouseClicked {
                pos, inside: true, ..
            } if self.clickable => pos,
            _ => return Handled::Bubble,
        };

        let rect = ctx.rect();

        let start = self.axis.main(rect.left_top());
        let end = self.axis.main(rect.right_bottom() - 1);
        let pos = self.axis.main(pos);

        let value = inverse_lerp(start, end, pos).unwrap_or(0.0);
        self.value = denormalize(value, self.range.clone());

        self.changed = true;
        Handled::Sink
    }

    fn layout(&mut self, mut layout: Layout, space: Space) -> Size {
        let main = self.axis.main(layout.property(Slider::SIZE));
        let size = self.axis.pack(main, 1.0);
        space.fit(size)
    }

    fn draw(&mut self, mut render: Render) {
        let pixel = render.property(Slider::TRACK).resolve(self.axis);
        let track = render.color(Slider::TRACK_COLOR);
        render.surface.fill_with(Pixel::new(pixel).fg(track));

        let extent: f32 = self.axis.main(render.rect().size());
        let value = normalize(self.value, self.range.clone());
        let x = lerp(0.0, extent - 1.0, value);
        let pos: Pos2 = self.axis.pack(x, 0.0);

        let knob = render.property(Slider::KNOB);
        let knob_color = render.color(Slider::KNOB_COLOR);

        let knob_color = if render.is_hovered() {
            render.theme.secondary
        } else {
            knob_color
        };

        render.surface.set(pos, Pixel::new(knob).fg(knob_color));
    }
}
