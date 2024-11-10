use core::f32;

use crate::{
    layout::{Axis, CrossAlign, Justify},
    math::{remap, vec2, Pos2, Rect, Size, Space, Vec2},
    view::{
        Builder, Elements, EventCtx, Handled, Interest, Layout, Palette, Render, StyleKind, Ui,
        View, ViewEvent,
    },
    Key, Pixel, Rgba,
};

#[derive(Debug, Default)]
struct ListState {
    flex: f32,
    main: Vec<f32>,
    cross: Vec<f32>,
}

impl ListState {
    const fn new() -> Self {
        Self {
            flex: 0.0,
            main: Vec::new(),
            cross: Vec::new(),
        }
    }

    fn resize(&mut self, len: usize) {
        self.main.resize(len, 0.0);
        self.cross.resize(len, 0.0);
    }

    fn main_sum(&self) -> f32 {
        self.main.iter().copied().sum()
    }

    fn cross_sum(&self) -> f32 {
        self.cross.iter().copied().fold(0.0, f32::max)
    }
}

#[derive(Debug)]
struct ListParams {
    max_major: f32,
    min_minor: f32,
    max_minor: f32,
    total_gap: f32,
}

// TODO move the scrolling stuff on this so other types can also do scrolling
#[derive(Default)]
struct ScrollState {
    scrollable: bool,
    pos: usize,
    knob_held: bool,
}

pub type ScrollClass = fn(&Palette, Axis) -> ScrollStyle;

#[derive(Copy, Clone)]
pub struct ScrollStyle {
    pub knob: char,
    pub knob_grab: Option<char>,
    pub track: Option<char>,
    pub track_color: Option<Rgba>,
    pub knob_color: Rgba,
    pub knob_grab_color: Option<Rgba>,
    pub background: Rgba,
}

impl ScrollStyle {
    pub fn default(palette: &Palette, axis: Axis) -> Self {
        Self {
            knob: axis.main((
                Elements::THICK_HORIZONTAL_LINE,
                Elements::THICK_VERTICAL_LINE,
            )),
            knob_grab: Some(axis.main((
                Elements::MEDIUM_RECT, //
                Elements::LARGE_RECT,
            ))),
            track: Some(axis.main((
                Elements::DASH_HORIZONTAL_LINE, //
                Elements::DASH_VERTICAL_LINE,
            ))),
            track_color: None,
            knob_color: palette.primary,
            knob_grab_color: Some(palette.secondary),
            background: palette.background,
        }
    }
}

#[must_use = "a view does nothing unless `show()` or `show_children()` is called"]
pub struct List {
    axis: Axis,
    justify: Justify,
    cross_align: CrossAlign,
    gap: f32,
    state: ListState,
    scroll: ScrollState,
    class: StyleKind<ScrollClass, ScrollStyle>,
}

impl List {
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

    pub const fn justify(mut self, justify: Justify) -> Self {
        self.justify = justify;
        self
    }

    pub const fn cross_align(mut self, cross_align: CrossAlign) -> Self {
        self.cross_align = cross_align;
        self
    }

    pub const fn gap(mut self, gap: i32) -> Self {
        self.gap = gap as f32;
        self
    }

    pub const fn scrollable(mut self, scrollable: bool) -> Self {
        self.scroll.scrollable = scrollable;
        self
    }

    pub const fn class(mut self, class: ScrollClass) -> Self {
        self.class = StyleKind::Deferred(class);
        self
    }

    pub const fn style(mut self, style: ScrollStyle) -> Self {
        self.class = StyleKind::Direct(style);
        self
    }
}

impl List {
    fn draw_scrollbar(&mut self, render: &mut Render) {
        if self.total_extent() <= render.rect().height() as f32 {
            return;
        }

        let style = match self.class {
            StyleKind::Deferred(style) => (style)(render.palette, self.axis),
            StyleKind::Direct(style) => style,
        };

        let rect = render.local_rect();
        let extent = self.axis.cross(rect.right_bottom() - 1);

        // TODO track

        let track = style.track.unwrap_or(' ');
        let track_color = style.track_color.unwrap_or(render.palette.outline);

        let bar_rect = Rect::from_min_size(self.axis.pack(0, extent), rect.size());
        render
            .surface
            .fill_rect(bar_rect, style.background)
            .fill_rect_with(bar_rect, Pixel::new(track).fg(track_color));

        let pos: Pos2 = self.axis.pack(self.knob_index(rect), extent);
        let hovered =
            self.scroll.knob_held || (render.mouse_pos() == pos + render.rect().left_top());

        let knob = if hovered {
            style.knob_grab.unwrap_or(style.knob)
        } else {
            style.knob
        };

        let color = if hovered {
            style.knob_grab_color.unwrap_or(style.knob_color)
        } else {
            style.knob_color
        };

        render.surface.set(pos, Pixel::new(knob).fg(color));
    }

    fn knob_offset(&self, size: Vec2) -> i32 {
        let total = self.total_extent() - self.axis.main::<f32>(size);
        let extent = self.axis.main(size - 1);
        remap(self.scroll.pos as f32, 0.0..=total, 0.0..=extent).round() as i32
    }

    fn knob_index(&self, rect: Rect) -> i32 {
        let total = self.total_extent() - self.axis.main::<f32>(rect.size());
        let size = rect.right_bottom() - 1;
        let extent: f32 = self.axis.main(size);
        remap(self.scroll.pos as f32, 0.0..=total, 0.0..=extent).round() as i32
    }

    fn scroll(&mut self, delta: i32, rect: Rect) {
        let total = self.total_extent() as usize;
        let max = total.saturating_sub(self.axis.main::<i32>(rect.size()) as usize);
        let new = self.scroll.pos.saturating_add_signed(delta as isize);
        self.scroll.pos = new.clamp(0, max);
    }

    fn total_extent(&self) -> f32 {
        let gap = (self.state.main.len() as f32 - 1.0) * self.gap;
        (self.state.main_sum() + gap).round()
    }

    #[cfg_attr(feature = "profile", profiling::function)]
    fn flex_layout(&mut self, layout: &mut Layout, args: ListParams) {
        self.state.flex = 0.0;

        let node = layout.nodes.get_current();
        // non-flex stuff
        for (i, &child) in node.children.iter().enumerate() {
            let flex = layout.flex(child);
            if flex.has_flex() {
                self.state.flex += flex.factor();
                self.state.main[i] = 0.0;
                continue;
            }

            let space = Space::new(
                self.axis.pack(0.0, args.min_minor),
                self.axis.pack(f32::INFINITY, args.max_minor),
            );

            let size = layout.compute(child, space);
            self.state.main[i] = self.axis.main(size);
            self.state.cross[i] = self.axis.cross(size);
        }

        // flex stuff
        let remaining = f32::max(args.max_major - args.total_gap - self.state.main_sum(), 0.0);
        let division = (remaining / self.state.flex).floor();
        // assert!(division.is_finite());

        for (i, &child) in node.children.iter().enumerate() {
            let flex = layout.flex(child);
            if !flex.has_flex() {
                continue;
            }

            if flex.is_expand() {
                continue;
            }

            let major = division * flex.factor();
            let space = Space::new(
                self.axis.pack(0.0, args.min_minor),
                self.axis.pack(major, args.max_minor),
            );

            let size = layout.compute(child, space);
            self.state.main[i] = self.axis.main(size);
            self.state.cross[i] = self.axis.cross(size);
        }

        // expanded stuff
        let remaining = f32::max(args.max_major - args.total_gap - self.state.main_sum(), 0.0);
        let division = (remaining / self.state.flex).floor();
        // assert!(division.is_finite());

        for (i, &child) in node.children.iter().enumerate() {
            let flex = layout.flex(child);
            if !flex.has_flex() {
                continue;
            }

            if !flex.is_expand() {
                continue;
            }

            let major = division * flex.factor();
            let space = Space::new(
                self.axis.pack(major, args.min_minor),
                self.axis.pack(major, args.max_minor),
            );

            let size = layout.compute(child, space);
            self.state.main[i] = self.axis.main(size);
            self.state.cross[i] = self.axis.cross(size);
        }
    }
}

impl std::fmt::Debug for List {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("List")
            .field("axis", &self.axis)
            .field("main_spacing", &self.justify)
            .field("cross_align", &self.cross_align)
            .field("gap", &self.gap)
            // .field("state", &self.state)
            .finish()
    }
}

impl<'v> Builder<'v> for List {
    type View = Self;
}

impl View for List {
    type Args<'v> = Self;
    type Response = ();

    fn create(args: Self::Args<'_>) -> Self {
        args
    }

    fn update(&mut self, args: Self::Args<'_>, _: &Ui) -> Self::Response {
        *self = Self {
            state: std::mem::take(&mut self.state),
            scroll: ScrollState {
                pos: self.scroll.pos,
                knob_held: self.scroll.knob_held,
                ..args.scroll
            },
            ..args
        }
    }

    fn primary_axis(&self) -> Axis {
        self.axis
    }

    fn interests(&self) -> Interest {
        if self.scroll.scrollable {
            Interest::MOUSE | Interest::FOCUS_INPUT
        } else {
            Interest::NONE
        }
    }

    fn event(&mut self, event: ViewEvent, ctx: EventCtx) -> Handled {
        if !self.scroll.scrollable {
            return Handled::Bubble;
        }

        let rect = ctx.rect();

        let extent = match self.axis {
            Axis::Horizontal => rect.left_bottom() - vec2(0, 1),
            Axis::Vertical => rect.right_top() - vec2(1, 0),
        };

        let knob: Pos2 = self.axis.pack(self.knob_offset(rect.size()), 0);
        let knob_pos = knob + extent;

        match event {
            ViewEvent::KeyInput { key, .. } => {
                let delta = match key {
                    Key::Up => -1,
                    Key::Down => 1,
                    Key::PageUp => self.axis.main(-rect.size()),
                    Key::PageDown => self.axis.main(rect.size()),
                    Key::Home => i32::MIN,
                    Key::End => i32::MAX,
                    _ => return Handled::Bubble,
                };
                self.scroll(delta, rect);
                Handled::Sink
            }

            ViewEvent::MouseMove { pos, .. } => {
                if knob_pos != pos {
                    self.scroll.knob_held = false;
                }
                Handled::Sink
            }

            ViewEvent::MouseHeld {
                pos, inside: true, ..
            } => {
                self.scroll.knob_held = knob_pos == pos;
                Handled::Sink
            }

            ViewEvent::MouseDrag {
                current,
                inside: true,
                ..
            } if self.scroll.knob_held => {
                let max = self.total_extent() - self.axis.main::<f32>(rect.size()).abs();

                let main = self.axis.main((rect.left(), rect.top()));
                let delta: i32 = self.axis.main(current - main);
                let extent: i32 = self.axis.main(rect.size() - 1);

                self.scroll.pos = remap(
                    delta as f32, //
                    0.0..=extent as f32,
                    0.0..=max,
                ) as usize;

                Handled::Sink
            }

            ViewEvent::MouseDrag {
                delta,
                inside: true,
                modifiers,
                ..
            } => {
                let scale = if modifiers.is_ctrl() { 3 } else { 1 };
                self.scroll(self.axis.main::<i32>(-delta) * scale, rect);
                Handled::Sink
            }

            ViewEvent::MouseScroll { delta, modifiers } => {
                // TODO modifiers
                let scale = if modifiers.is_ctrl() { 3 } else { 1 };
                self.scroll(self.axis.main::<i32>(delta) * scale, rect);
                Handled::Sink
            }

            _ => Handled::Bubble,
        }
    }

    fn layout(&mut self, mut layout: Layout, mut space: Space) -> Size {
        if self.scroll.scrollable {
            layout.enable_clipping();
        }

        let margin: Size = if self.scroll.scrollable {
            self.axis.pack(0.0, 1.0)
        } else {
            Size::ZERO
        };

        space.max -= margin;

        let total_extent = self.axis.main(space.max);

        let node = layout.nodes.get_current();
        self.state.resize(node.children.len());

        let (min_major, min_minor) = self.axis.unpack(space.min);
        let (max_major, max_minor) = self.axis.unpack(space.max);

        let min_major = min_major.min(max_major);
        let min_minor = min_minor.min(max_minor);

        let total_gap = self.gap * (node.children.len() as f32 - 1.0);

        let align = self.cross_align;
        if align.is_fill() || (align.is_stretch() && min_minor == max_minor) {
            let args = ListParams {
                max_major,
                min_minor: max_minor,
                max_minor,
                total_gap,
            };
            self.flex_layout(&mut layout, args);
        } else {
            let args = ListParams {
                max_major,
                min_minor: 0.0,
                max_minor,
                total_gap,
            };
            self.flex_layout(&mut layout, args);

            if align.is_stretch() {
                let minor = f32::clamp(self.state.cross_sum(), min_minor, max_minor);
                let args = ListParams {
                    max_major,
                    min_minor: minor,
                    max_minor: minor,
                    total_gap,
                };
                self.flex_layout(&mut layout, args);
            }
        }

        let mut main = f32::clamp(self.state.main_sum() + total_gap, min_major, max_major);
        let cross = f32::clamp(self.state.cross_sum(), min_minor, max_minor);

        for (i, child_main) in self
            .justify
            .layout(&self.state.main, main, self.gap)
            .enumerate()
        {
            let child_cross = self.cross_align.align(cross, self.state.cross[i]);
            let offset: Pos2 = self.axis.pack(
                child_main - self.scroll.pos as f32, //
                child_cross,
            );

            let node = node.children[i];
            if self.axis.main::<f32>(offset) >= total_extent {
                layout.remove(node);
            } else {
                layout.set_position(node, offset);
            }
        }

        if main.is_infinite() {
            main = self.state.main_sum();
        }

        let size: Size = self.axis.pack(main, cross);
        size + margin
    }

    fn draw(&mut self, mut render: Render) {
        if !self.scroll.scrollable {
            self.default_draw(render);
            return;
        }

        let current = render.nodes.get_current();

        for &child in &current.children {
            if !render.layout.contains(child) {
                break;
            }
            render.draw(child)
        }

        self.draw_scrollbar(&mut render);
    }
}

pub const fn list() -> List {
    List {
        axis: Axis::Horizontal,
        justify: Justify::Start,
        cross_align: CrossAlign::Start,
        gap: 0.0,
        state: ListState::new(),
        scroll: ScrollState {
            scrollable: false,
            pos: 0,
            knob_held: false,
        },
        class: StyleKind::deferred(ScrollStyle::default),
    }
}
