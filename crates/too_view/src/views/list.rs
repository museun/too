use too_math::layout::Axis;

use crate::{
    geom::{Size, Space, Vector},
    response::UserResponse,
    view::Context,
    views::AxisExt as _,
    LayoutCtx, NoResponse, UpdateCtx, View, ViewExt,
};

#[derive(Copy, Clone, Default)]
pub enum MainSpacing {
    #[default]
    Start,
    End,
    Center,
    SpaceBetween,
    SpaceAround,
    SpaceEvenly,
}

impl MainSpacing {
    pub fn layout(self, sizes: &[f32], size: f32, gap: f32) -> impl Iterator<Item = f32> + '_ {
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

#[derive(Copy, Clone, Default)]
pub enum CrossAlign {
    #[default]
    Start,
    End,
    Center,
    Stretch,
    Fill,
}

impl CrossAlign {
    pub fn align(self, available: f32, size: f32) -> f32 {
        match self {
            Self::Start => 0.0,
            Self::End => available - size,
            Self::Center => (available - size) * 0.5,
            Self::Stretch => 0.0,
            Self::Fill => 0.0,
        }
    }
}

#[derive(Copy, Clone)]
pub struct ListParams {
    axis: Axis,
    main_spacing: MainSpacing,
    cross_align: CrossAlign,
    gap: f32,
}

impl ListParams {
    pub const fn horizontal() -> Self {
        Self::axis(Axis::Horizontal)
    }

    pub const fn vertical() -> Self {
        Self::axis(Axis::Vertical)
    }

    pub const fn axis(axis: Axis) -> Self {
        Self {
            axis,
            main_spacing: MainSpacing::Start,
            cross_align: CrossAlign::Start,
            gap: 0.0,
        }
    }

    pub const fn gap(mut self, gap: f32) -> Self {
        self.gap = gap;
        self
    }

    pub const fn main_spacing(mut self, main_spacing: MainSpacing) -> Self {
        self.main_spacing = main_spacing;
        self
    }

    pub const fn cross_align(mut self, cross_align: CrossAlign) -> Self {
        self.cross_align = cross_align;
        self
    }
}

// where would we store this?
#[derive(Default)]
struct ListState {
    // flex: f32,
    main: Vec<f32>,
    cross: Vec<f32>,
}

impl ListState {
    fn new(len: usize) -> Self {
        Self {
            // flex: 0.0,
            main: vec![0.0; len],
            cross: vec![0.0; len],
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
        self.cross.iter().copied().fold(0.0, |a, c| a.max(c))
    }
}

struct List {
    state: ListState,
    params: ListParams,
}

impl<T: 'static> View<T> for List {
    type Args<'a> = ListParams;
    type Response = NoResponse;

    fn create(args: Self::Args<'_>) -> Self {
        Self {
            state: ListState::default(),
            params: args,
        }
    }

    fn update(&mut self, ctx: UpdateCtx<T>, args: Self::Args<'_>) -> Self::Response {
        self.params = args;
    }

    fn layout(&mut self, mut ctx: LayoutCtx<T>, space: Space) -> Size {
        fn stack_layout<T: 'static>(
            stack: &mut List,
            ctx: &mut LayoutCtx<T>,
            main_max: f32,
            cross_min: f32,
            cross_max: f32,
            gap: f32,
        ) {
            // stack.state.flex = 0.0;
            for i in 0..ctx.children.len() {
                // if child is flex
                //  flex += flex amount
                //  stack.state.main[i] = 0.0;
                //  continue

                let space = Space::new(
                    stack.params.axis.pack(0.0, cross_min),
                    stack.params.axis.pack(f32::INFINITY, main_max),
                );

                let size = ctx.compute_layout(ctx.children[i], space);
                stack.state.main[i] = stack.params.axis.major(size);
                stack.state.cross[i] = stack.params.axis.minor(size);
            }

            // let remaining = f32::max(main_max - gap - stack.state.main_sum(), 0.0);
            // let division = remaining / stack.state.flex; // inf if no flex

            // for i in 0..ctx.children.len() {
            //     // do flex layout
            // }
        }

        let space = space.loosen();

        self.state.resize(ctx.children.len());

        let (main_min, main_max) = self.params.axis.unpack(space.min);
        let (cross_min, cross_max) = self.params.axis.unpack(space.max);

        let main_min = main_min.min(main_max);
        let cross_min = cross_min.min(cross_max);

        let total_gap = self.params.gap * (ctx.children.len() as f32 - 1.0);

        let align = self.params.cross_align;
        // TODO this is very ugly
        if matches!(align, CrossAlign::Fill)
            || (matches!(align, CrossAlign::Start) && cross_min == cross_max)
        {
            stack_layout(self, &mut ctx, main_max, cross_max, cross_max, total_gap);
        } else {
            stack_layout(self, &mut ctx, main_max, 0.0, cross_max, total_gap);
            if matches!(align, CrossAlign::Stretch) {
                let cross = self.state.cross_sum().clamp(cross_min, cross_max);
                stack_layout(self, &mut ctx, main_max, cross, cross, total_gap);
            }
        }

        // let main = f32::clamp(self.state.main_sum() + total_gap, main_min, main_max);
        // let cross = f32::clamp(self.state.cross_sum(), cross_min, cross_max);

        let main = self.state.main_sum() + total_gap;
        let cross = self.state.cross_sum();

        for (i, child_main) in self
            .params
            .main_spacing
            .layout(&self.state.main, main, self.params.gap)
            .enumerate()
        {
            let child_cross = self.params.cross_align.align(cross, self.state.cross[i]);
            let offset: Vector = self.params.axis.pack(child_main, child_cross);
            ctx.translate_pos(ctx.children[i], offset);
        }

        self.params.axis.pack(main, cross)
    }
}

pub fn row<T: 'static, R>(
    ctx: &mut Context<T>,
    show: impl FnOnce(&mut Context<T>) -> R,
) -> UserResponse<R> {
    list(ListParams::axis(Axis::Horizontal), ctx, show)
}

pub fn column<T: 'static, R>(
    ctx: &mut Context<T>,
    show: impl FnOnce(&mut Context<T>) -> R,
) -> UserResponse<R> {
    list(ListParams::axis(Axis::Vertical), ctx, show)
}

pub fn list<T: 'static, R>(
    params: impl Into<ListParams>,
    ctx: &mut Context<T>,
    show: impl FnOnce(&mut Context<T>) -> R,
) -> UserResponse<R> {
    let args = params.into();
    List::show_children(args, ctx, show)
}
