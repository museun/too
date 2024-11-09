use core::f32;
use std::ops::RangeInclusive;

use crate::{
    layout::Axis,
    math::{denormalize, inverse_lerp, lerp, normalize, Pos2, Size, Space},
    view::{
        Builder, Elements, EventCtx, Handled, Interest, Knob, Layout, Palette, Render, StyleKind,
        Ui, View, ViewEvent,
    },
    Pixel, Rgba,
};

pub type SliderClass = fn(&Palette, Axis) -> SliderStyle;

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct SliderStyle {
    pub track_color: Rgba,
    pub knob_color: Rgba,
    pub track_hovered: Option<Rgba>,
    pub knob_hovered: Option<Rgba>,
    pub knob: char,
    pub track: char,
}

impl SliderStyle {
    pub fn default(palette: &Palette, axis: Axis) -> Self {
        Self {
            track_color: palette.outline,
            knob_color: palette.primary,
            track_hovered: None,
            knob_hovered: None,
            knob: axis.main((Knob::MEDIUM, Knob::LARGE)),
            track: axis.main((
                Elements::THICK_HORIZONTAL_LINE,
                Elements::THICK_VERTICAL_LINE,
            )),
        }
    }

    pub fn small_rounded(palette: &Palette, axis: Axis) -> Self {
        Self {
            knob: Knob::ROUND,
            track: axis.main((Elements::HORIZONTAL_LINE, Elements::VERTICAL_LINE)),
            ..Self::default(palette, axis)
        }
    }

    pub fn small_diamond(palette: &Palette, axis: Axis) -> Self {
        Self {
            knob: Knob::DIAMOND,
            track: axis.main((Elements::HORIZONTAL_LINE, Elements::VERTICAL_LINE)),
            ..Self::default(palette, axis)
        }
    }

    pub fn small_square(palette: &Palette, axis: Axis) -> Self {
        Self {
            knob: Knob::SMALL,
            track: axis.main((Elements::HORIZONTAL_LINE, Elements::VERTICAL_LINE)),
            ..Self::default(palette, axis)
        }
    }

    pub fn large(palette: &Palette, axis: Axis) -> Self {
        Self {
            knob: Knob::LARGE,
            track: Elements::MEDIUM_RECT,
            ..Self::default(palette, axis)
        }
    }

    pub fn large_filled(palette: &Palette, axis: Axis) -> Self {
        Self {
            knob: Knob::LARGE,
            track: Elements::LARGE_RECT,
            ..Self::default(palette, axis)
        }
    }
}

pub fn slider(value: &mut f32) -> Slider {
    Slider {
        value,
        range: 0.0..=1.0,
        clickable: true,
        axis: Axis::Horizontal,
        class: StyleKind::Deferred(SliderStyle::small_rounded),
    }
}

// TODO step by
#[must_use = "a view does nothing unless `show()` or `show_children()` is called"]
pub struct Slider<'v> {
    value: &'v mut f32,
    range: RangeInclusive<f32>,
    clickable: bool,
    axis: Axis,
    class: StyleKind<SliderClass, SliderStyle>,
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

    pub const fn class(mut self, class: SliderClass) -> Self {
        self.class = StyleKind::Deferred(class);
        self
    }

    pub const fn style(mut self, style: SliderStyle) -> Self {
        self.class = StyleKind::Direct(style);
        self
    }
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
    class: StyleKind<SliderClass, SliderStyle>,
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
            class: args.class,
        }
    }

    fn update(&mut self, args: Self::Args<'_>, ui: &Ui) -> Self::Response {
        self.range = args.range.clone();
        self.clickable = args.clickable;
        self.axis = args.axis;
        self.class = args.class;

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

    fn layout(&mut self, layout: Layout, space: Space) -> Size {
        let main = self.axis.main((20.0, 1.0));
        let size = self.axis.pack(main, 1.0);
        space.fit(size)
    }

    fn draw(&mut self, mut render: Render) {
        let style = match self.class {
            StyleKind::Deferred(style) => (style)(render.palette, self.axis),
            StyleKind::Direct(style) => style,
        };

        let track_color = if render.is_hovered() {
            style.track_hovered.unwrap_or(style.track_color)
        } else {
            style.track_color
        };

        render
            .surface
            .fill_with(Pixel::new(style.track).fg(track_color));

        let extent: f32 = self.axis.main(render.rect().size());
        let value = normalize(self.value, self.range.clone());
        let x = lerp(0.0, extent - 1.0, value);
        let pos: Pos2 = self.axis.pack(x, 0.0);

        let knob_color = if render.is_hovered() {
            style.knob_hovered.unwrap_or(style.knob_color)
        } else {
            style.knob_color
        };

        render
            .surface
            .set(pos, Pixel::new(style.knob).fg(knob_color));
    }
}
