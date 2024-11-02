use crate::{
    layout::Axis,
    math::{pos2, remap, vec2, Pos2, Rect, Vec2},
    view::{
        geom::{Size, Space},
        Builder, Elements, EventCtx, Handled, Interest, Layout, Render, Ui, View, ViewEvent,
    },
    Key, Pixel,
};

#[derive(Copy, Clone, Debug, Default, PartialEq)]
enum ScrollAxis {
    Horizontal,
    #[default]
    Vertical,
    Both,
}

impl ScrollAxis {
    const fn horizontal(self, using: bool) -> Self {
        match self {
            Self::Vertical if using => Self::Both,
            Self::Horizontal | Self::Both if !using => Self::Vertical,
            _ => self,
        }
    }

    const fn vertical(self, using: bool) -> Self {
        match self {
            Self::Horizontal if using => Self::Both,
            Self::Vertical | Self::Both if !using => Self::Horizontal,
            _ => self,
        }
    }
}

#[derive(Debug, Default)]
pub struct Scrollable {
    axis: ScrollAxis,
}

impl Scrollable {
    pub const fn horizontal(mut self, horizontal: bool) -> Self {
        self.axis = self.axis.horizontal(horizontal);
        self
    }

    pub const fn vertical(mut self, vertical: bool) -> Self {
        self.axis = self.axis.vertical(vertical);
        self
    }

    const fn has_horizontal(&self) -> bool {
        matches!(self.axis, ScrollAxis::Horizontal | ScrollAxis::Both)
    }

    const fn has_vertical(&self) -> bool {
        matches!(self.axis, ScrollAxis::Vertical | ScrollAxis::Both)
    }
}

impl<'v> Builder<'v> for Scrollable {
    type View = ScrollView;
}

#[derive(Debug, Default)]
#[must_use = "a view does nothing unless `show()` or `show_children()` is called"]
pub struct ScrollView {
    opts: Scrollable,

    // TODO don't store these inverted
    pos: Vec2,

    child_size: Vec2,
    size: Vec2,

    x_hovered: bool,
    y_hovered: bool,
}

impl ScrollView {
    fn find_knob(&self, size: Vec2, axis: Axis) -> Pos2 {
        let max: f32 = axis.main(self.child_size);
        let (min, cross) = axis.unpack(self.size);
        let pos = axis.main(self.pos);
        let main = remap(pos, 0.0..=max, 0.0..=min);
        axis.pack(main, cross)
    }

    fn find_knob_horizontal(&self, size: Vec2) -> Option<Pos2> {
        if !self.opts.has_horizontal() {
            return None;
        }

        // (self.x_pos != 0).then_some(pos2(x, size.y))
        Some(self.find_knob(size, Axis::Horizontal))
    }

    fn find_knob_vertical(&self, size: Vec2) -> Option<Pos2> {
        if !self.opts.has_vertical() {
            return None;
        }

        // (self.y_pos != 0).then_some(pos2(size.x, y))
        Some(self.find_knob(size, Axis::Vertical))
    }

    fn draw_scrollbars(&self, render: &mut Render, local: Rect) {
        let size = local.size() - 1;

        let showing_horizontal = self.find_knob_horizontal(size);
        if let Some(pos) = showing_horizontal {
            let rect = Rect::from_min_size(
                local.left_bottom() - pos2(0, 1), //
                vec2(local.width(), 1),
            );
            let pixel = Elements::DASH_HORIZONTAL_LINE;
            let pixel = Pixel::new(pixel)
                .fg(render.theme.outline)
                .bg(render.theme.surface);
            render.surface.fill_rect_with(rect, pixel);

            let pixel = if self.x_hovered {
                Elements::MEDIUM_RECT
            } else {
                Elements::THICK_HORIZONTAL_LINE
            };
            let pixel = Pixel::new(pixel).fg(render.theme.contrast);
            render.surface.set(pos, pixel);
        }

        let showing_vertical = self.find_knob_vertical(size);
        if let Some(pos) = showing_vertical {
            let rect = Rect::from_min_size(
                local.right_top() - pos2(1, 0), //
                vec2(1, local.height()),
            );
            let pixel = Elements::DASH_VERTICAL_LINE;
            let pixel = Pixel::new(pixel)
                .fg(render.theme.outline)
                .bg(render.theme.surface);
            render.surface.fill_rect_with(rect, pixel);

            let pixel = if self.y_hovered {
                Elements::LARGE_RECT
            } else {
                Elements::THICK_VERTICAL_LINE
            };
            let pixel = Pixel::new(pixel).fg(render.theme.contrast);
            render.surface.set(pos, pixel);
        };

        if let (Some(h), Some(v)) = (showing_horizontal, showing_vertical) {
            const MERGE: char = '┘';
            const BOTH_SELECTED: char = '┛';
            const PARTIAL_HORIZONATAL: char = '╸';
            const PARTIAL_VERTICAL: char = '╹';

            let bottom_right = local.right_bottom() - 1;

            let cell = Pixel::new(MERGE)
                .fg(render.theme.outline)
                .bg(render.theme.surface);
            render.surface.set(bottom_right, cell);

            if h == bottom_right {
                let cell = Pixel::new(PARTIAL_HORIZONATAL).fg(render.theme.contrast);
                render.surface.set(h, cell);
            }

            if v == bottom_right {
                let cell = Pixel::new(PARTIAL_VERTICAL).fg(render.theme.contrast);
                render.surface.set(v, cell);
            }

            if h == v {
                let cell = Pixel::new(BOTH_SELECTED).fg(render.theme.contrast);
                render.surface.set(h, cell);
            }
        }
    }
}

impl View for ScrollView {
    type Args<'v> = Scrollable;
    type Response = ();

    fn create(args: Self::Args<'_>) -> Self {
        Self {
            opts: args,
            ..Default::default()
        }
    }

    fn update(&mut self, args: Self::Args<'_>, ui: &Ui) -> Self::Response {
        self.opts = args;
    }

    fn primary_axis(&self) -> Axis {
        match self.opts.axis {
            ScrollAxis::Horizontal => Axis::Horizontal,
            ScrollAxis::Vertical | ScrollAxis::Both => Axis::Vertical,
        }
    }

    fn interests(&self) -> Interest {
        Interest::MOUSE | Interest::FOCUS_INPUT
    }

    fn event(&mut self, event: ViewEvent, ctx: EventCtx) -> Handled {
        let rect = ctx.rect();
        let offset = rect.left_top();

        let delta = match event {
            ViewEvent::KeyInput { key, modifiers } => match key {
                Key::Up => vec2(0, -1),
                Key::Down => vec2(0, 1),
                Key::Left => vec2(-1, 0),
                Key::Right => vec2(1, 0),

                Key::PageUp if modifiers.is_ctrl() => vec2(-self.size.x, 0),
                Key::PageDown if modifiers.is_ctrl() => vec2(self.size.x, 0),
                Key::PageUp => vec2(0, -self.size.y),
                Key::PageDown => vec2(0, self.size.y),

                Key::Home if modifiers.is_ctrl() => Vec2::MIN_X,
                Key::End if modifiers.is_ctrl() => Vec2::MAX_X,
                Key::Home => Vec2::MIN_Y,
                Key::End => Vec2::MAX_Y,

                _ => return Handled::Bubble,
            },

            ViewEvent::MouseScroll { delta, modifiers } if modifiers.is_ctrl_only() => {
                vec2(delta.y, 0)
            }

            ViewEvent::MouseScroll { delta, .. } => vec2(0, -delta.y),

            // slider knobs
            ViewEvent::MouseMove { pos, .. } => {
                let horizontal = self.find_knob_horizontal(rect.size() - vec2(1, 0));
                let vertical = self.find_knob_vertical(rect.size() - vec2(0, 1));
                self.y_hovered = vertical == Some(pos - offset + vec2(1, 0));
                self.x_hovered = horizontal == Some(pos - offset + vec2(0, 1));
                return Handled::Bubble;
            }

            ViewEvent::MouseDrag {
                current,
                modifiers,
                inside: true,
                ..
            } if self.y_hovered => {
                let (start, end) = (rect.top() as f32, rect.bottom() as f32 - 1.0);
                let max = self.child_size.y as f32;
                let pos = current.y as f32;

                let y = match pos {
                    pos if pos <= start => i32::MIN,
                    pos if pos >= end => i32::MAX,
                    pos => {
                        let inverse = (pos - start) / (end - start) * (max - end);
                        let pos = inverse.round() as i32;

                        self.pos.y = -pos;
                        self.pos.y = self.pos.y.min(0).max(-(max.abs() as i32 - self.size.y));
                        return Handled::Sink;
                    }
                };

                vec2(0, y)
            }

            ViewEvent::MouseDrag {
                current,
                modifiers,
                inside: true,
                ..
            } if self.x_hovered => {
                let (start, end) = (rect.left() as f32, rect.right() as f32 - 1.0);
                let max = self.child_size.x as f32;
                let pos = current.x as f32;

                let x = match pos {
                    pos if pos <= start => i32::MIN,
                    pos if pos >= end => i32::MAX,
                    pos => {
                        let inverse = (pos - start) / (end - start) * (max - end);
                        let pos = inverse.round() as i32;
                        self.pos.x = -pos;
                        self.pos.x = self.pos.x.min(0).max(-(max.abs() as i32 - self.size.x));
                        return Handled::Sink;
                    }
                };

                vec2(x, 0)
            }

            ViewEvent::MouseDrag {
                delta,
                modifiers,
                inside: true,
                ..
            } => {
                if modifiers.is_alt() {
                    -delta
                } else if modifiers.is_ctrl_only() {
                    vec2(delta.x, 0)
                } else {
                    vec2(0, delta.y)
                }
            }

            _ => return Handled::Bubble,
        };

        let Vec2 { x, y } = self.child_size;

        if self.opts.has_horizontal() {
            self.pos.x = self
                .pos
                .x
                .saturating_sub(delta.x)
                .min(0)
                .max(-(x.abs() - self.size.x));
        }

        if self.opts.has_vertical() {
            self.pos.y = self
                .pos
                .y
                .saturating_sub(delta.y)
                .min(0)
                .max(-(y.abs() - self.size.y));
        }

        Handled::Sink
    }

    fn layout(&mut self, mut layout: Layout, space: Space) -> Size {
        layout.enable_clipping();
        let node = layout.nodes.get_current();

        let child_space = Space::new(space.max, Size::INFINITY);
        let mut size = Size::ZERO;
        for &child in &node.children {
            size = size.max(layout.compute(child, child_space));
            layout.set_position(child, self.pos);
        }

        self.child_size = size.into();
        space.max
    }

    fn draw(&mut self, mut render: Render) {
        let local = render.local_rect();
        self.size = local.size();

        let node = render.nodes.get_current();
        for &child in &node.children {
            render.draw(child)
        }

        self.draw_scrollbars(&mut render, local);
    }
}

pub const fn scrollable() -> Scrollable {
    Scrollable {
        axis: ScrollAxis::Vertical,
    }
}
