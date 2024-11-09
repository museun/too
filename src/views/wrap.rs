use std::ops::Range;

use crate::{
    layout::{Align, Axis, Justify},
    math::Pos2,
    view::{
        geom::{Size, Space},
        Builder, Layout, Ui, View,
    },
};

#[derive(Copy, Clone, Debug, Default, PartialEq)]
pub struct Wrap {
    axis: Axis,
    main_justify: Justify,
    cross_justify: Justify,
    cross_align: Align,
    row_gap: f32,
    column_gap: f32,
}

impl Wrap {
    pub const fn new(axis: Axis) -> Self {
        Self {
            axis,
            main_justify: Justify::Start,
            cross_justify: Justify::Start,
            cross_align: Align::START,
            row_gap: 0.0,
            column_gap: 0.0,
        }
    }

    pub const fn main_justify(mut self, main_justify: Justify) -> Self {
        self.main_justify = main_justify;
        self
    }

    pub const fn cross_justify(mut self, cross_justify: Justify) -> Self {
        self.cross_justify = cross_justify;
        self
    }

    pub const fn gap(self, gap: i32) -> Self {
        self.row_gap(gap).column_gap(gap)
    }

    pub const fn row_gap(mut self, row_gap: i32) -> Self {
        self.row_gap = row_gap as f32;
        self
    }

    pub const fn column_gap(mut self, column_gap: i32) -> Self {
        self.column_gap = column_gap as f32;
        self
    }

    pub const fn cross_align(mut self, align: Align) -> Self {
        self.cross_align = align;
        self
    }

    pub const fn horizontal() -> Self {
        Self::new(Axis::Horizontal)
    }

    pub const fn vertical() -> Self {
        Self::new(Axis::Vertical)
    }
}

impl<'v> Builder<'v> for Wrap {
    type View = WrapView;
}

#[derive(Debug)]
pub struct WrapView {
    wrap: Wrap,
    state: WrapState,
}

impl View for WrapView {
    type Args<'v> = Wrap;
    type Response = ();

    fn create(args: Self::Args<'_>) -> Self {
        Self {
            wrap: args,
            state: WrapState::default(),
        }
    }

    fn update(&mut self, args: Self::Args<'_>, ui: &Ui) -> Self::Response {
        self.wrap = args;
    }

    fn primary_axis(&self) -> Axis {
        self.wrap.axis
    }

    fn layout(&mut self, mut layout: Layout, space: Space) -> Size {
        let node = layout.nodes.get_current();

        self.state.resize(node.children.len());

        let (min_main, min_cross) = self.wrap.axis.unpack(space.min);
        let (max_main, max_cross) = self.wrap.axis.unpack(space.max);

        for i in 0..node
            .children
            .len()
            // FIXME this limits the wrappable to the parent rect size
            //  with infinite views (scrollable) this'll have to be smart
            .min(space.max.width as usize * space.max.height as usize)
        {
            let size = layout.compute(node.children[i], Space::UNBOUNDED);
            self.state.main[i] = self.wrap.axis.main(size);
        }

        let (main_gap, cross_gap) = self
            .wrap
            .axis
            .unpack((self.wrap.row_gap, self.wrap.column_gap));

        let mut main = 0.0;
        self.state.runs.clear();
        self.state.cross.clear();

        let mut run_start = 0;
        let mut run_main = 0.0;
        let mut run_cross = 0.0;

        for i in 0..node.children.len() {
            let size = layout.size(node.children[i]);
            let (child_main, child_cross) = self.wrap.axis.unpack(size);

            let gap = if run_main > 0.0 { main_gap } else { 0.0 };
            if run_main + child_main + gap <= max_main {
                run_main += gap + child_main;
                run_cross = f32::max(run_cross, child_cross);
                continue;
            }

            self.state.runs.push(run_start..i);
            self.state.cross.push(run_cross);
            main = f32::max(main, run_main);

            run_start = i;
            run_main = child_main;
            run_cross = child_cross;
        }

        self.state.runs.push(run_start..node.children.len());
        self.state.cross.push(run_cross);
        main = f32::max(main, run_main);

        let total_gap = cross_gap * (self.state.runs.len() as f32 - 1.0);

        let main = f32::clamp(main, min_main, max_main);
        let cross = f32::clamp(self.state.cross() + total_gap, min_cross, max_cross);

        for (i, pos) in self
            .wrap
            .cross_justify
            .layout(&self.state.cross, cross, cross_gap)
            .enumerate()
        {
            let run = self.state.runs[i].clone();
            let run_cross = self.state.cross[i];

            for (child, j) in self
                .wrap
                .main_justify
                .layout(&self.state.main[run.clone()], main, main_gap)
                .zip(run)
            {
                let child_cross = self.wrap.axis.cross(layout.size(node.children[j]));
                let child_align = self.wrap.cross_align.align(run_cross, child_cross);
                let offset: Pos2 = self.wrap.axis.pack(child, pos + child_align);
                layout.set_position(node.children[j], offset);
            }
        }

        self.wrap.axis.pack(main, cross)
    }
}

#[derive(Default, Debug)]
struct WrapState {
    main: Vec<f32>,
    runs: Vec<Range<usize>>,
    cross: Vec<f32>,
}

impl WrapState {
    pub const fn new() -> Self {
        Self {
            main: Vec::new(),
            runs: Vec::new(),
            cross: Vec::new(),
        }
    }

    fn resize(&mut self, len: usize) {
        self.main.resize(len, 0.0);
    }

    fn cross(&self) -> f32 {
        self.cross.iter().copied().sum()
    }
}

pub const fn horizontal_wrap() -> Wrap {
    Wrap::horizontal()
}

pub const fn vertical_wrap() -> Wrap {
    Wrap::vertical()
}
