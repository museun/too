use core::f32;

use crate::{
    layout::Axis,
    math::Pos2,
    view::{
        geom::{Size, Space},
        Builder, Layout, Ui, View,
    },
};

#[derive(Copy, Clone, Debug, Default, PartialEq)]
pub enum Justify {
    #[default]
    Start,
    End,
    Center,
    SpaceBetween,
    SpaceAround,
    SpaceEvenly,
}

impl Justify {
    pub fn layout(self, sizes: &[f32], size: f32, gap: f32) -> impl Iterator<Item = f32> + use<'_> {
        let count = sizes.len() as f32;
        let total_gap = gap * (count - 1.0);
        let total_size = sizes.iter().sum::<f32>() + total_gap;

        let gap = match self {
            Self::Start | Self::End | Self::Center => gap,
            Self::SpaceBetween => (size - (total_size - total_gap)) / (count - 1.0),
            Self::SpaceAround => (size - (total_size - total_gap)) / count,
            Self::SpaceEvenly => (size - (total_size - total_gap)) / (count + 1.0),
        };

        let mut pos = match self {
            Self::Start | Self::SpaceBetween => 0.0,
            Self::Center => (size - total_size) * 0.5,
            Self::End => size - total_size,
            Self::SpaceAround => gap * 0.5,
            Self::SpaceEvenly => gap,
        };

        let mut iter = sizes.iter();
        std::iter::from_fn(move || {
            let old = pos;
            pos += *iter.next()? + gap;
            Some(old)
        })
    }
}

#[derive(Copy, Clone, Debug, Default, PartialEq)]
pub enum CrossAlign {
    #[default]
    Start,
    End,
    Center,
    Stretch,
    Fill,
}

impl CrossAlign {
    pub const fn is_stretch(&self) -> bool {
        matches!(self, Self::Stretch)
    }

    pub const fn is_fill(&self) -> bool {
        matches!(self, Self::Fill)
    }

    pub fn align(self, available: f32, size: f32) -> f32 {
        match self {
            Self::Start | Self::Stretch | Self::Fill => 0.0,
            Self::End => available - size,
            Self::Center => (available - size) * 0.5,
        }
    }
}

#[derive(Debug, Default)]
struct ListState {
    flex: f32,
    main: Vec<f32>, // this is all we need. start at 'pos' and take until we've reached the rect extent
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

#[derive(Default)]
#[must_use = "a view does nothing unless `show()` or `show_children()` is called"]
pub struct List {
    axis: Axis,
    justify: Justify,
    cross_align: CrossAlign,
    gap: f32,
    state: ListState,
}

impl List {
    fn flex_layout(&mut self, layout: &mut Layout, args: ListParams) {
        let node = layout.nodes.get_current();
        self.state.flex = 0.0;

        // non-flex stuff
        for i in 0..node.children.len() {
            let flex = layout.flex(node.children[i]);
            if flex.has_flex() {
                self.state.flex += flex.factor();
                self.state.main[i] = 0.0;
                continue;
            }

            let space = Space::new(
                self.axis.pack(0.0, args.min_minor),
                self.axis.pack(f32::INFINITY, args.max_minor),
                // self.axis.pack(args.max_major, args.max_minor),
            );

            let size = layout.compute(node.children[i], space);
            self.state.main[i] = self.axis.main(size);
            self.state.cross[i] = self.axis.cross(size);
        }

        // expanded stuff
        let remaining = f32::max(args.max_major - args.total_gap - self.state.main_sum(), 0.0);
        let division = remaining / self.state.flex;
        // assert!(division.is_finite());

        for i in 0..node.children.len() {
            let flex = layout.flex(node.children[i]);
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

            let size = layout.compute(node.children[i], space);
            self.state.main[i] = self.axis.main(size);
            self.state.cross[i] = self.axis.cross(size);
        }

        // flex stuff
        let remaining = f32::max(args.max_major - args.total_gap - self.state.main_sum(), 0.0);
        let division = remaining / self.state.flex;
        // assert!(division.is_finite());

        for i in 0..node.children.len() {
            let flex = layout.flex(node.children[i]);
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

            let size = layout.compute(node.children[i], space);
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

    fn update(&mut self, args: Self::Args<'_>, ui: &Ui) -> Self::Response {
        *self = Self {
            state: std::mem::take(&mut self.state),
            ..args
        }
    }

    fn primary_axis(&self) -> Axis {
        self.axis
    }

    // fn flex(&self) -> Flex {
    //     Flex::Tight(1.0)
    // }

    // fn event(&mut self, event: ViewEvent, ctx: EventCtx) -> Handled {
    //     let Some(knob) = self.find_knob() else {
    //         return Handled::Bubble;
    //     };
    //     let knob = knob + self.rect.left_top();

    //     let delta = match event {
    //         ViewEvent::MouseMove { pos, .. } => {
    //             self.knob_hovered = pos == knob;
    //             return Handled::Sink;
    //         }

    //         ViewEvent::KeyInput { key, .. } => match key {
    //             Key::Up => vec2(0, 1),
    //             Key::Down => vec2(0, -1),
    //             Key::Left => vec2(1, 0),
    //             Key::Right => vec2(-1, 0),
    //             Key::PageUp => self.rect.size(),
    //             Key::PageDown => -self.rect.size(),
    //             Key::Home => Vec2::MAX,
    //             Key::End => Vec2::MIN,
    //             _ => return Handled::Bubble,
    //         },
    //         // horizontal scroll is kind of difficult to do on a scrollwheel
    //         ViewEvent::MouseScroll { delta, .. } => Vec2::splat(-delta.y),
    //         ViewEvent::MouseDrag { delta, current, .. } if !self.knob_hovered => delta,

    //         ViewEvent::MouseDrag { current, .. } => {
    //             let Scrollable::Enabled(pos) = &mut self.scrollable else {
    //                 return Handled::Bubble;
    //             };

    //             let current: f32 = self.axis.main(current);

    //             let (start, end): (f32, f32) = (
    //                 self.axis.main(self.rect.left_top()),
    //                 self.axis.main(self.rect.right_bottom()),
    //             );

    //             let end = end - 1.0;
    //             if current <= start {
    //                 *pos = 0;
    //                 return Handled::Sink;
    //             }

    //             if current >= end {
    //                 *pos = (self.state.main.len() as i32 - 1 - end as i32).abs();
    //                 return Handled::Sink;
    //             }

    //             let max = self.state.main.len() as f32;
    //             let new = (current.clamp(start, end) - start) / (end - start) * (max - end);
    //             let new = new.floor() as i32;
    //             *pos = new.clamp(0, (max as i32 - end as i32).abs());

    //             return Handled::Sink;
    //         }

    //         _ => return Handled::Bubble,
    //     };

    //     let delta = self.axis.main(delta);
    //     let max = self.state.main.len() as i32;

    //     let extent = self.axis.main(self.rect.size());
    //     self.scrollable.scroll(delta, max, extent);

    //     Handled::Sink
    // }

    fn layout(&mut self, mut layout: Layout, space: Space) -> Size {
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
            let offset: Pos2 = self.axis.pack(child_main, child_cross);
            let node = node.children[i];
            layout.set_position(node, offset);
        }

        if main.is_infinite() {
            main = self.state.main_sum();
        }

        self.axis.pack(main, cross)
    }
}

pub const fn list() -> List {
    List {
        axis: Axis::Horizontal,
        justify: Justify::Start,
        cross_align: CrossAlign::Start,
        gap: 0.0,
        state: ListState::new(),
    }
}
