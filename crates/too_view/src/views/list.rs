use too::layout::Axis;

use crate::{
    geom::{Size, Space, Vector},
    view::Context,
    LayoutCtx, UpdateCtx, View, ViewExt,
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
    pub const fn is_start(&self) -> bool {
        matches!(self, Self::Start)
    }

    pub const fn is_end(&self) -> bool {
        matches!(self, Self::End)
    }

    pub const fn is_center(&self) -> bool {
        matches!(self, Self::Center)
    }

    pub const fn is_stretch(&self) -> bool {
        matches!(self, Self::Stretch)
    }

    pub const fn is_fill(&self) -> bool {
        matches!(self, Self::Fill)
    }

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

// TODO space evenly isn't counting the gap
#[derive(Copy, Clone)]
pub struct List {
    axis: Axis,
    main_spacing: MainSpacing,
    cross_align: CrossAlign,
    gap: f32,
}

impl List {
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

    pub fn show<T: 'static, R>(
        self,
        ctx: &mut Context<T>,
        show: impl FnOnce(&mut Context<T>) -> R,
    ) -> R {
        let (_, resp) = ListView::show_children(self, ctx, show);
        resp
    }
}

#[derive(Default)]
struct ListState {
    flex: f32,
    main: Vec<f32>,
    cross: Vec<f32>,
}

impl ListState {
    fn new(len: usize) -> Self {
        Self {
            flex: 0.0,
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

struct ListArgs {
    max_major: f32,
    min_minor: f32,
    max_minor: f32,
    total_gap: f32,
}

struct ListView {
    state: ListState,
    params: List,
}

impl ListView {
    fn flex_layout<T: 'static>(&mut self, ctx: &mut LayoutCtx<T>, args: ListArgs) {
        self.state.flex = 0.0;

        // non-flex stuff
        for i in 0..ctx.children.len() {
            if let Some(flex) = ctx.properties.flex(ctx.children[i]) {
                self.state.flex += flex.amount;
                self.state.main[i] = 0.0;
                continue;
            }

            let space = Space::new(
                self.params.axis.pack(0.0, args.min_minor),
                self.params.axis.pack(f32::INFINITY, args.max_minor),
            );

            let size = ctx.compute_layout(ctx.children[i], space);
            self.state.main[i] = self.params.axis.main(size);
            self.state.cross[i] = self.params.axis.cross(size);
        }

        // expanded stuff
        let remaining = f32::max(args.max_major - args.total_gap - self.state.main_sum(), 0.0);
        let division = remaining / self.state.flex; // inf if no flex

        for i in 0..ctx.children.len() {
            let Some(flex) = ctx.properties.flex(ctx.children[i]) else {
                continue;
            };
            if !flex.tight {
                continue;
            }

            let major = division * flex.amount;
            let space = Space::new(
                self.params.axis.pack(0.0, args.min_minor),
                self.params.axis.pack(major, args.max_minor),
            );

            let size = ctx.compute_layout(ctx.children[i], space);
            self.state.main[i] = self.params.axis.main(size);
            self.state.cross[i] = self.params.axis.cross(size);
        }

        // flex stuff
        let remaining = f32::max(args.max_major - args.total_gap - self.state.main_sum(), 0.0);
        let division = remaining / self.state.flex;

        for i in 0..ctx.children.len() {
            let Some(flex) = ctx.properties.flex(ctx.children[i]) else {
                continue;
            };
            if flex.tight {
                continue;
            }

            let major = division * flex.amount;
            let space = Space::new(
                self.params.axis.pack(major, args.min_minor),
                self.params.axis.pack(major, args.max_minor),
            );

            let size = ctx.compute_layout(ctx.children[i], space);
            self.state.main[i] = self.params.axis.main(size);
            self.state.cross[i] = self.params.axis.cross(size);
        }
    }
}

impl<T: 'static> View<T> for ListView {
    type Args<'a> = List;
    type Response = ();

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
        self.state.resize(ctx.children.len());

        let (min_major, min_minor) = self.params.axis.unpack(space.min);
        let (max_major, max_minor) = self.params.axis.unpack(space.max);

        let min_major = min_major.min(max_major);
        let min_minor = min_minor.min(max_minor);

        let total_gap = self.params.gap * (ctx.children.len() as f32 - 1.0);

        let align = self.params.cross_align;
        if align.is_fill() || (align.is_stretch() && min_minor == max_minor) {
            let args = ListArgs {
                max_major,
                min_minor: max_minor,
                max_minor,
                total_gap,
            };
            self.flex_layout(&mut ctx, args);
        } else {
            let args = ListArgs {
                max_major,
                min_minor: 0.0,
                max_minor,
                total_gap,
            };
            self.flex_layout(&mut ctx, args);

            if align.is_stretch() {
                let minor = f32::clamp(self.state.cross_sum(), min_minor, max_minor);
                let args = ListArgs {
                    max_major,
                    min_minor: minor,
                    max_minor: minor,
                    total_gap,
                };
                self.flex_layout(&mut ctx, args);
            }
        }

        let main = f32::clamp(self.state.main_sum() + total_gap, min_major, max_major);
        let cross = f32::clamp(self.state.cross_sum(), min_minor, max_minor);

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

pub fn row<T: 'static, R>(ctx: &mut Context<T>, show: impl FnOnce(&mut Context<T>) -> R) -> R {
    list(List::axis(Axis::Horizontal), ctx, show)
}

pub fn column<T: 'static, R>(ctx: &mut Context<T>, show: impl FnOnce(&mut Context<T>) -> R) -> R {
    list(List::axis(Axis::Vertical), ctx, show)
}

pub fn list<T: 'static, R>(
    params: impl Into<List>,
    ctx: &mut Context<T>,
    show: impl FnOnce(&mut Context<T>) -> R,
) -> R {
    params.into().show(ctx, show)
}

#[cfg(test)]
mod tests {
    use std::collections::VecDeque;

    use too::{animation::AnimationManager, math::vec2, overlay::Overlay, Rgba};

    use crate::{
        debug_fmt::short_name, geom::Rectf, view::Context, views::*, Node, Properties, Ui, ViewId,
    };

    // #[test]
    // fn constrain_it() {
    //     let space = Space::new(Size::new(0.0, 25.0), Size::new(f32::INFINITY, 0.0));
    //     space.constrain(other)
    // }

    #[test]
    fn list_thing() {
        struct DebugNode<T: 'static> {
            id: ViewId,
            name: String,
            rect: Rectf,
            children: Vec<Self>,
            _marker: std::marker::PhantomData<T>,
        }

        impl<T: 'static> DebugNode<T> {
            fn new(root: ViewId, nodes: &thunderdome::Arena<Node<T>>) -> Self {
                fn build<T: 'static>(
                    nodes: &mut Vec<DebugNode<T>>,
                    id: ViewId,
                    view_nodes: &thunderdome::Arena<Node<T>>,
                ) {
                    let mut children = vec![];
                    for &child in &(*view_nodes[id.0]).children {
                        build(&mut children, child, view_nodes)
                    }

                    nodes.push(DebugNode {
                        id,
                        name: short_name((*view_nodes[id.0]).view.type_name()),
                        rect: (*view_nodes[id.0]).rect,
                        children,
                        _marker: std::marker::PhantomData,
                    });
                }

                let mut children = vec![];
                build(&mut children, (*nodes[root.0]).children[0], &nodes);

                Self {
                    id: root,
                    name: short_name((*nodes[root.0]).view.type_name()),
                    rect: (*nodes[root.0]).rect,
                    children,
                    _marker: std::marker::PhantomData,
                }
            }

            fn render(&self) -> String {
                fn print<T: 'static>(
                    children: &[DebugNode<T>],
                    prefix: &str,
                    out: &mut impl std::fmt::Write,
                ) {
                    for (i, node) in children.iter().enumerate() {
                        if i < children.len() - 1 {
                            _ = writeln!(
                                out,
                                "{prefix}- {}({:?}): {},{},{},{}",
                                node.name,
                                node.id,
                                node.rect.min.x,
                                node.rect.min.y,
                                node.rect.max.x,
                                node.rect.max.y
                            );
                            print(&node.children, &format!("{prefix}|"), out);
                        } else if i < children.len()
                            && children.last().filter(|c| c.children.is_empty()).is_some()
                        {
                            _ = writeln!(
                                out,
                                "{prefix}-\\ {}({:?}): {},{},{},{}",
                                node.name,
                                node.id,
                                node.rect.min.x,
                                node.rect.min.y,
                                node.rect.max.x,
                                node.rect.max.y
                            );
                            print(&node.children, &format!("{prefix} "), out);
                        } else {
                            _ = writeln!(
                                out,
                                "{prefix}\\ {}({:?}): {},{},{},{}",
                                node.name,
                                node.id,
                                node.rect.min.x,
                                node.rect.min.y,
                                node.rect.max.x,
                                node.rect.max.y
                            );
                            print(&node.children, &format!("{prefix} "), out);
                        }
                    }
                }

                use std::fmt::Write as _;
                let mut out = String::new();
                let _ = writeln!(
                    out,
                    "{}({:?}): {},{},{},{}",
                    self.name,
                    self.id,
                    self.rect.min.x,
                    self.rect.min.y,
                    self.rect.max.x,
                    self.rect.max.y
                );
                print(&self.children, "", &mut out);
                out
            }
        }

        #[derive(Default)]
        struct State {
            r: f32,
            g: f32,
            b: f32,
        }

        let mut ui = <Ui<State>>::new((80.0, 25.0), Properties::default());

        let make_it = |ctx: &mut Context<State>| {
            List::horizontal().show(ctx, |ctx| {
                flex(ctx, |ctx| fill(ctx, Rgba::hex("#F00")));
                flex(ctx, |ctx| {
                    column(ctx, |ctx| {
                        slider(ctx, |ctx| SliderParams::new(&mut ctx.r));
                        slider(ctx, |ctx| SliderParams::new(&mut ctx.g));
                        slider(ctx, |ctx| SliderParams::new(&mut ctx.b));
                    });
                });
            });
        };

        macro_rules! ctx {
            () => {
                too::Context {
                    overlay: &mut Overlay::default(),
                    commands: &mut VecDeque::default(),
                    size: vec2(80, 25),
                    animations: &mut AnimationManager::default(),
                }
            };
        }

        let mut state = State::default();
        ui.scope(&mut state, make_it, ctx!());
        ui.scope(&mut state, make_it, ctx!());

        let out = DebugNode::new(ui.root, &ui.nodes).render();
        eprintln!("{out}")
    }
}
