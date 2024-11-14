use std::time::Duration;

use crate::{
    animation::{easing, Animation},
    layout::Axis,
    math::{lerp, Pos2, Size, Space},
    view::{
        Builder, Elements, EventCtx, Handled, Interest, Layout, Palette, Render, StyleKind, Ui,
        View, ViewEvent,
    },
    Pixel, Rgba,
};

#[derive(Debug, Copy, Clone)]
pub struct ToggleStyle {
    pub track: char,
    pub track_color: Rgba,
    pub track_hovered: Option<Rgba>,

    pub on_knob: char,
    pub on_knob_color: Rgba,

    pub off_knob: char,
    pub off_knob_color: Rgba,

    pub on_knob_hovered: Option<Rgba>,
    pub off_knob_hovered: Option<Rgba>,
}

impl ToggleStyle {
    pub fn default(palette: &Palette, axis: Axis, toggled: bool) -> Self {
        Self {
            track: axis.main((Elements::MEDIUM_RECT, Elements::LARGE_RECT)),
            track_color: palette.outline,
            track_hovered: None,
            on_knob: Elements::LARGE_RECT,
            on_knob_color: palette.primary,
            off_knob: Elements::LARGE_RECT,
            off_knob_color: palette.secondary,
            on_knob_hovered: None,
            off_knob_hovered: None,
        }
    }

    pub fn large(palette: &Palette, axis: Axis, toggled: bool) -> Self {
        Self::default(palette, axis, toggled)
    }

    pub fn small_rounded(palette: &Palette, axis: Axis, toggled: bool) -> Self {
        Self {
            track: axis.main((
                Elements::THICK_HORIZONTAL_LINE,
                Elements::THICK_VERTICAL_LINE,
            )),
            on_knob: Elements::CIRCLE,
            off_knob: Elements::CIRCLE,
            ..Self::default(palette, axis, toggled)
        }
    }

    pub fn small_diamond(palette: &Palette, axis: Axis, toggled: bool) -> Self {
        Self {
            track: axis.main((
                Elements::THICK_HORIZONTAL_LINE,
                Elements::THICK_VERTICAL_LINE,
            )),
            on_knob: Elements::DIAMOND,
            off_knob: Elements::DIAMOND,
            ..Self::default(palette, axis, toggled)
        }
    }

    pub fn small_square(palette: &Palette, axis: Axis, toggled: bool) -> Self {
        Self {
            track: axis.main((
                Elements::THICK_HORIZONTAL_LINE,
                Elements::THICK_VERTICAL_LINE,
            )),
            on_knob: Elements::MEDIUM_RECT,
            off_knob: Elements::MEDIUM_RECT,
            ..Self::default(palette, axis, toggled)
        }
    }
}

pub type ToggleClass = fn(&Palette, Axis, bool) -> ToggleStyle;

#[derive(Copy, Clone, Debug, PartialEq, Default)]
pub struct ToggleResponse {
    changed: bool,
}

impl ToggleResponse {
    pub fn changed(&self) -> bool {
        self.changed
    }
}

#[must_use = "a view does nothing unless `show()` or `show_children()` is called"]
pub struct ToggleSwitch<'a> {
    value: &'a mut bool,
    axis: Axis,
    class: StyleKind<ToggleClass, ToggleStyle>,
}

impl<'a> ToggleSwitch<'a> {
    pub fn new(value: &'a mut bool) -> Self {
        Self {
            value,
            axis: Axis::Horizontal,
            class: StyleKind::deferred(ToggleStyle::default),
        }
    }

    pub fn horizontal(mut self) -> Self {
        self.axis = Axis::Horizontal;
        self
    }

    pub fn vertical(mut self) -> Self {
        self.axis = Axis::Vertical;
        self
    }

    pub fn axis(mut self, axis: Axis) -> Self {
        self.axis = axis;
        self
    }

    pub const fn class(mut self, class: ToggleClass) -> Self {
        self.class = StyleKind::Deferred(class);
        self
    }

    pub const fn style(mut self, style: ToggleStyle) -> Self {
        self.class = StyleKind::Direct(style);
        self
    }
}

impl<'v> Builder<'v> for ToggleSwitch<'v> {
    type View = ToggleSwitchView;
}

#[derive(Debug)]
pub struct ToggleSwitchView {
    value: bool,
    changed: bool,
    axis: Axis,
    class: StyleKind<ToggleClass, ToggleStyle>,
}

impl View for ToggleSwitchView {
    type Args<'v> = ToggleSwitch<'v>;
    type Response = ToggleResponse;

    fn create(args: Self::Args<'_>) -> Self {
        Self {
            value: *args.value,
            changed: false,
            axis: args.axis,
            class: args.class,
        }
    }

    fn update(&mut self, args: Self::Args<'_>, _: &Ui) -> Self::Response {
        self.axis = args.axis;
        self.class = args.class;

        let changed = self.changed;
        if std::mem::take(&mut self.changed) {
            *args.value = self.value;
        } else if self.value != *args.value {
            self.value = *args.value;
        };
        ToggleResponse { changed }
    }

    fn interests(&self) -> Interest {
        Interest::MOUSE_INSIDE
    }

    fn event(&mut self, event: ViewEvent, ctx: EventCtx) -> Handled {
        match event {
            ViewEvent::MouseClicked { pos, .. } => {
                self.value = !self.value;
                self.changed = true;

                ctx.animation.add_once(ctx.current(), || {
                    Animation::new()
                        .oneshot(true)
                        .with(easing::sine_in_out)
                        .schedule(Duration::from_millis(150))
                        .unwrap()
                });
            }

            ViewEvent::MouseDrag { delta, .. }
                if (self.value && self.axis.main::<i32>(delta).is_negative())
                    || (!self.value && self.axis.main::<i32>(delta).is_positive()) =>
            {
                self.value = !self.value;
                self.changed = true;

                ctx.animation.add_once(ctx.current(), || {
                    Animation::new()
                        .oneshot(true)
                        .with(easing::sine_in_out)
                        .schedule(Duration::from_millis(50))
                        .unwrap()
                });
            }

            _ => return Handled::Bubble,
        };

        Handled::Sink
    }

    fn layout(&mut self, _: Layout, space: Space) -> Size {
        let main = self.axis.main((4.0, 2.0));
        space.fit(self.axis.pack(main, 1.0))
    }

    fn draw(&mut self, mut render: Render) {
        let rect = render.surface.rect();

        let selected = self.value;

        let style = match self.class {
            StyleKind::Deferred(style) => (style)(render.palette, self.axis, selected),
            StyleKind::Direct(style) => style,
        };

        let color = if render.is_hovered() {
            style.track_hovered.unwrap_or(style.track_color)
        } else {
            style.track_color
        };

        render.surface.fill_with(Pixel::new(style.track).fg(color));

        let extent = self.axis.main::<f32>(rect.size()) - 1.0;

        let x = match render.animation.get_mut(render.current) {
            Some(animation) if selected => lerp(0.0, extent, *animation.value),
            Some(animation) if !selected => lerp(extent, 0.0, *animation.value),
            _ if selected => extent,
            _ => 0.0,
        };

        let color = match (render.is_hovered(), selected) {
            (true, true) => style.on_knob_hovered.unwrap_or(style.on_knob_color),
            (true, false) => style.off_knob_hovered.unwrap_or(style.off_knob_color),
            (false, true) => style.on_knob_color,
            (false, false) => style.off_knob_color,
        };

        let knob = if selected {
            style.on_knob
        } else {
            style.off_knob
        };

        let pos: Pos2 = self.axis.pack(x, 0.0);
        render.surface.set(pos, Pixel::new(knob).fg(color));
    }
}

pub fn toggle_switch(value: &mut bool) -> ToggleSwitch<'_> {
    ToggleSwitch::new(value)
}
