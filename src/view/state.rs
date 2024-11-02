use std::{
    any::TypeId,
    borrow::{Borrow, BorrowMut},
    cell::{Ref, RefCell, RefMut},
    collections::VecDeque,
};

use compact_str::{CompactString, ToCompactString};
use thunderdome::Arena;

use crate::{
    layout::{Anchor2, Axis, LinearLayout},
    math::{Pos2, Rect, Vec2},
    AnimationManager, Rgba, Surface, Text,
};

use super::{
    geom::{Size, Space},
    helpers::{ArenaDebug, Queue},
    input::InputState,
    layout::IntrinsicSize,
    style::{Stylesheet, Theme},
    ui::Ui,
    view::Erased,
    CroppedSurface, Interest, Layout, Render, View,
};

#[derive(Copy, Clone, Debug, Default, PartialEq, PartialOrd, Eq, Ord, Hash)]
pub enum DebugMode {
    PerFrame,
    #[default]
    Rolling,
    Off,
}

#[derive(Debug, Default)]
pub(in crate::view) struct Debug {
    queue: RefCell<Queue<CompactString>>,
    pub(in crate::view) mode: std::cell::Cell<DebugMode>,
}

thread_local! {
    static DEBUG: Debug = const { Debug::new() }
}

pub fn debug(msg: impl ToCompactString) {
    DEBUG.with(|c| c.push(msg))
}

impl Debug {
    const fn new() -> Self {
        Self {
            queue: RefCell::new(Queue::new(25)),
            mode: std::cell::Cell::new(DebugMode::Rolling),
        }
    }

    pub(in crate::view) fn for_each(mut f: impl FnMut(&str)) {
        DEBUG.with(|c| {
            for msg in c.queue.borrow().iter() {
                f(msg);
            }
        })
    }

    fn push(&self, msg: impl ToCompactString) {
        if matches!(self.mode.get(), DebugMode::Off) {
            return;
        }
        let msg = msg.to_compact_string();
        let msg = msg.trim();
        if msg.is_empty() {
            return;
        }
        self.queue.borrow_mut().push(msg.into());
    }

    fn iter(&mut self) -> impl ExactSizeIterator<Item = &str> + use<'_> {
        self.queue.get_mut().iter().map(<_>::as_ref)
    }
}

pub struct State {
    pub(in crate::view) nodes: ViewNodes,
    pub(in crate::view) layout: LayoutNodes,
    pub(in crate::view) render: RenderNodes,
    pub(in crate::view) input: InputState,
    pub(in crate::view) animations: AnimationManager,
    pub(in crate::view) theme: Theme,
    pub(in crate::view) stylesheet: Stylesheet,
    pub(in crate::view) frame_count: u64,
}

impl Default for State {
    fn default() -> Self {
        Self::new()
    }
}

impl State {
    pub fn new() -> Self {
        let nodes = ViewNodes::new();
        let mut layout = LayoutNodes::new();
        layout.nodes.insert_at(nodes.root.0, LayoutNode::default());

        Self {
            nodes,
            layout,
            render: RenderNodes::new(),
            input: InputState::default(),
            animations: AnimationManager::new(),
            theme: Theme::dark(),
            stylesheet: Stylesheet::default(),
            frame_count: 0,
        }
    }

    pub fn debug(&self, msg: impl ToCompactString) {
        if matches!(DEBUG.with(|c| c.mode.get()), DebugMode::Off) {
            return;
        }
        let msg = msg.to_compact_string();
        let msg = msg.trim();
        if msg.is_empty() {
            return;
        }
        debug(msg);
    }

    pub fn set_debug_mode(&self, mode: DebugMode) {
        DEBUG.with(|c| c.mode.set(mode))
    }

    pub fn root(&self) -> ViewId {
        self.nodes.root()
    }

    pub fn current(&self) -> ViewId {
        self.nodes.current()
    }

    pub fn update(&mut self, dt: f32) {
        self.animations.update(dt);
    }

    #[profiling::function]
    pub fn event(&mut self, event: &crate::Event) {
        if let crate::Event::Resize(size) = event {
            DEBUG.with(|c| c.queue.borrow_mut().resize(size.y as usize))
        }

        // TODO debounce 'event'
        let _resp = self
            .input
            .update(&self.nodes, &self.layout, &mut self.animations, event);
    }

    #[profiling::function]
    pub fn render(&mut self, surface: &mut Surface) {
        self.frame_count += 1;
        let root = self.root();
        surface.clear(surface.rect(), self.theme.background);

        self.render.draw_all(
            &self.nodes, //
            &self.layout,
            &self.input,
            &mut self.animations,
            &mut self.stylesheet,
            &self.theme,
            surface,
        );

        self.stylesheet.reset();

        DEBUG.with(|c| {
            let mut debug = c.queue.borrow_mut();
            if debug.is_empty() {
                return;
            }

            let mut layout = LinearLayout::vertical()
                .wrap(false)
                .anchor(Anchor2::RIGHT_TOP)
                .layout(surface.rect());

            match c.mode.get() {
                DebugMode::PerFrame => {
                    for msg in debug.drain() {
                        let text = Text::new(msg).fg(Rgba::hex("#F00")).bg(Rgba::hex("#000"));
                        if let Some(rect) = layout.allocate(text.size()) {
                            surface.text(rect, text);
                        }
                    }
                }
                DebugMode::Rolling => {
                    for msg in debug.iter() {
                        let text = Text::new(msg).fg(Rgba::hex("#F00")).bg(Rgba::hex("#000"));
                        if let Some(rect) = layout.allocate(text.size()) {
                            surface.text(rect, text);
                        }
                    }
                }
                DebugMode::Off => {}
            }
        });
    }

    #[profiling::function]
    pub fn build<R: 'static>(&mut self, rect: Rect, mut show: impl FnMut(&Ui) -> R) -> R {
        let root = self.nodes.root;
        self.layout.nodes[root.0].rect = rect;

        self.begin();
        let resp = show(&Ui::new(self));
        self.end();

        self.layout.compute_all(
            &self.nodes, //
            &self.input,
            &mut self.stylesheet,
            rect,
        );

        resp
    }

    fn begin(&mut self) {
        let root = self.nodes.root;
        self.nodes.nodes.get_mut()[root.0].next = 0;
        self.render.start();
        self.input
            .begin(&self.nodes, &self.layout, &mut self.animations);
    }

    fn end(&mut self) {
        for id in self.nodes.removed.get_mut().drain(..) {
            eprintln!("moreving node from layout: {id:?}");
            self.layout.nodes.remove(id.0);
        }
        self.input.end();
        self.layout.interest.clear();
    }
}

pub struct ViewNodes {
    nodes: RefCell<Arena<ViewNode>>,
    stack: RefCell<Vec<ViewId>>,
    removed: RefCell<Vec<ViewId>>,
    root: ViewId,
}

impl std::fmt::Debug for ViewNodes {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ViewNodes")
            .field("root", &self.root)
            .field("nodes", &ArenaDebug(&self.nodes.borrow()))
            .finish()
    }
}

impl ViewNodes {
    pub(super) fn new() -> Self {
        let mut nodes = Arena::new();
        let root = nodes.insert(ViewNode {
            parent: None,
            children: Vec::new(),
            next: 0,
            view: RefCell::new(Slot::new(Root)),
        });

        Self {
            nodes: RefCell::new(nodes),
            stack: RefCell::default(),
            removed: RefCell::default(),
            root: ViewId(root),
        }
    }

    pub fn begin(&self, id: ViewId) {
        self.stack.borrow_mut().push(id);
    }

    pub fn end(&self, id: ViewId) {
        let Some(id) = self.stack.borrow_mut().pop() else {
            unreachable!("stack was empty");
        };
    }

    pub(in crate::view) fn begin_view<V: View>(
        &self,
        args: V::Args<'_>,
        ui: &Ui,
    ) -> (ViewId, V::Response) {
        let parent = self.current();
        let (id, resp) = self.update_view::<V>(parent, args, ui);
        self.stack.borrow_mut().push(id);
        (id, resp)
    }

    fn update_view<V: View>(
        &self,
        parent: ViewId,
        args: V::Args<'_>,
        ui: &Ui,
    ) -> (ViewId, V::Response) {
        let Some(id) = self.append_view(parent) else {
            return self.allocate_view::<V>(parent, args);
        };

        let type_id = self.nodes.borrow()[id.0].view.borrow().type_id();
        if type_id != TypeId::of::<V>() {
            self.remove_view(id);
            return self.allocate_view::<V>(parent, args);
        }

        self.nodes.borrow_mut()[id.0].next = 0;

        let mut view = self.nodes.borrow()[id.0].view.borrow_mut().take();
        let resp = {
            let Some(view) = view.as_mut_any().downcast_mut::<V>() else {
                unreachable!(
                    "type did not match: {} | {}",
                    view.type_name(),
                    std::any::type_name::<V>()
                );
            };
            view.update(args, ui)
        };

        self.nodes.borrow()[id.0].view.borrow_mut().give(view);
        (id, resp)
    }

    fn append_view(&self, parent: ViewId) -> Option<ViewId> {
        let parent = &mut self.nodes.borrow_mut()[parent.0];
        let id = parent.children.get(parent.next).copied()?;
        parent.next += 1;
        Some(id)
    }

    fn allocate_view<V: View>(&self, parent: ViewId, args: V::Args<'_>) -> (ViewId, V::Response) {
        let id = self.nodes.borrow_mut().insert(ViewNode {
            parent: Some(parent),
            children: vec![],
            next: 0,
            view: RefCell::new(Slot::new(V::create(args))),
        });

        let id = ViewId(id);

        let parent = &mut self.nodes.borrow_mut()[parent.0];
        if parent.next < parent.children.len() {
            parent.children[parent.next] = id;
        } else {
            parent.children.push(id);
        }
        parent.next += 1;

        (id, V::Response::default())
    }

    fn remove_view(&self, root: ViewId) {
        let mut queue = VecDeque::from_iter([root]);

        let mut nodes = self.nodes.borrow_mut();
        let mut removed = self.removed.borrow_mut();

        while let Some(id) = queue.pop_front() {
            removed.push(id);
            let Some(node) = nodes.remove(id.0) else {
                continue;
            };

            queue.extend(&node.children);
            let Some(parent) = node.parent else {
                continue;
            };
            let Some(parent) = nodes.get_mut(parent.0) else {
                continue;
            };
            let len = parent.children.len();
            parent.children.retain(|&child| child != id);

            let difference = len.abs_diff(parent.children.len());
            parent.next = parent.next.saturating_sub(difference);
        }
    }

    pub(in crate::view) fn end_view(&self, id: ViewId) {
        let Some(old) = self.stack.borrow_mut().pop() else {
            unreachable!("called end view without an active view")
        };
        assert_eq!(
            id, old,
            "end view ({id:?}) did not much begin view ({old:?})"
        );
        self.cleanup(id);
    }

    fn cleanup(&self, start: ViewId) {
        // why doesn't NLL drop this borrow at the 'return'?
        {
            let nodes = self.nodes.borrow();
            let node = &nodes[start.0];
            if node.next >= node.children.len() {
                return;
            }
        }

        let mut nodes = self.nodes.borrow_mut();
        let node = &mut nodes[start.0];

        let children = &node.children[node.next..];
        let mut queue = VecDeque::from_iter(children.iter().copied());
        node.children.truncate(node.next);

        let mut removed = self.removed.borrow_mut();
        while let Some(id) = queue.pop_front() {
            removed.push(id);
            let Some(next) = nodes.remove(id.0) else {
                unreachable!("child {id:?} should exist for {start:?}");
            };
            queue.extend(&next.children);
        }
    }

    pub const fn root(&self) -> ViewId {
        self.root
    }

    pub fn get(&self, id: ViewId) -> Option<Ref<'_, ViewNode>> {
        let nodes = self.nodes.borrow();
        Ref::filter_map(nodes, |nodes| nodes.get(id.0)).ok()
    }

    pub fn get_mut(&self, id: ViewId) -> Option<RefMut<'_, ViewNode>> {
        let nodes = self.nodes.borrow_mut();
        RefMut::filter_map(nodes, |nodes| nodes.get_mut(id.0)).ok()
    }

    // TODO use this in more places (any place that does `get_mut().unwrap().view.take()`)
    // TODO this should push the id to the stack and pop it off
    pub fn scoped<R>(&self, id: ViewId, mut act: impl FnMut(&mut dyn Erased) -> R) -> Option<R> {
        let nodes = self.nodes.borrow();
        let node = nodes.get(id.0)?;
        let mut view = node.view.borrow_mut().take();
        let resp = act(&mut *view);
        node.view.borrow_mut().give(view);
        Some(resp)
    }

    pub fn current(&self) -> ViewId {
        self.stack.borrow().last().copied().unwrap_or(self.root)
    }

    pub fn parent(&self) -> ViewId {
        self.stack
            .borrow()
            .iter()
            .nth_back(1)
            .copied()
            .unwrap_or(self.root)
    }

    pub fn get_current(&self) -> Ref<'_, ViewNode> {
        let index = self.current();
        let nodes = self.nodes.borrow();
        Ref::map(nodes, |nodes| &nodes[index.0])
    }
}

#[derive(Default)]
pub struct RenderNodes {
    clip_stack: Vec<Rect>,
    axis_stack: Vec<Axis>,
}

impl RenderNodes {
    const fn new() -> Self {
        Self {
            clip_stack: Vec::new(),
            axis_stack: Vec::new(),
        }
    }

    fn start(&mut self) {
        self.clip_stack.clear();
    }

    fn push_clip(&mut self, mut rect: Rect) {
        if let Some(prev) = self.clip_stack.last() {
            rect = rect.union(*prev)
        }
        self.clip_stack.push(rect);
    }

    fn pop_clip(&mut self) {
        assert!(self.clip_stack.pop().is_some())
    }

    #[profiling::function]
    pub fn draw_all(
        &mut self,
        nodes: &ViewNodes,
        layout: &LayoutNodes,
        input: &InputState,
        animation: &mut AnimationManager,
        stylesheet: &mut Stylesheet,
        theme: &Theme,
        surface: &mut Surface,
    ) {
        let surface = CroppedSurface {
            rect: surface.rect(),
            surface,
        };

        self.draw(
            nodes,
            layout,
            input,
            animation,
            stylesheet,
            theme,
            nodes.root(),
            surface,
        );
    }

    pub fn current_axis(&self) -> Option<Axis> {
        self.axis_stack.iter().nth_back(1).copied()
    }

    pub fn draw(
        &mut self,
        nodes: &ViewNodes,
        layout: &LayoutNodes,
        input: &InputState,
        animation: &mut AnimationManager,
        stylesheet: &mut Stylesheet,
        theme: &Theme,
        id: ViewId,
        surface: CroppedSurface,
    ) {
        let Some(node) = layout.nodes.get(id.0) else {
            return;
        };

        let mut rect = node.rect;
        if rect.width() == 0 || rect.height() == 0 {
            return;
        }

        // if let Some(pid) = nodes
        //     .get(id)
        //     .and_then(|node| node.parent)
        //     .filter(|&c| c != nodes.root)
        // {
        // if let Some(parent) = layout.nodes.get(pid.0) {
        //     if !parent.rect.contains_rect_inclusive(rect) {
        //         return;
        //     }
        // }
        // }

        if let Some(parent) = node.clipped_by {
            let Some(parent) = layout.nodes.get(parent.0) else {
                return;
            };
            if !rect.partial_contains_rect(parent.rect) {
                return;
            }
            rect = parent.rect.intersection(rect);
        }

        if node.clipping_enabled {
            self.push_clip(node.rect);
        }

        nodes.begin(id);
        stylesheet.swap(id);

        nodes
            .scoped(id, |node| {
                self.axis_stack.push(node.primary_axis());
                let surface = CroppedSurface {
                    rect,
                    surface: surface.surface,
                };
                let render = Render {
                    nodes,
                    layout,
                    animation,
                    surface,
                    stylesheet,
                    theme,
                    input,
                    render: self,
                };
                node.draw(render);
                self.axis_stack.pop();
            })
            .unwrap();

        stylesheet.swap(id);
        nodes.end(id);

        if node.clipping_enabled {
            self.pop_clip()
        }
    }
}

#[derive(Default, Debug)]
pub struct MouseInterest {
    layers: Vec<Vec<(ViewId, Interest)>>,
    stack: Vec<(ViewId, usize)>,
}

impl MouseInterest {
    pub const fn new() -> Self {
        Self {
            layers: Vec::new(),
            stack: Vec::new(),
        }
    }

    fn clear(&mut self) {
        self.layers.clear();
        self.stack.clear();
    }

    fn current_layer_root(&self) -> Option<ViewId> {
        self.stack.last().map(|&(id, _)| id)
    }

    fn push_layer(&mut self, id: ViewId) {
        let index = self.layers.len();
        self.layers.push(vec![]);
        self.stack.push((id, index));
    }

    fn pop_layer(&mut self) {
        assert!(self.stack.pop().is_some());
    }

    #[profiling::function]
    fn insert(&mut self, id: ViewId, interest: Interest) {
        self.stack
            .last()
            .and_then(|&(_, index)| self.layers.get_mut(index))
            .unwrap()
            .push((id, interest));
    }

    #[profiling::function]
    pub fn iter(&self) -> impl Iterator<Item = (ViewId, Interest)> + '_ {
        self.layers
            .iter()
            .rev()
            .flat_map(|layer| layer.iter().copied())
    }
}

#[derive(Default)]
pub struct LayoutNodes {
    nodes: Arena<LayoutNode>,
    clip_stack: Vec<ViewId>,
    axis_stack: Vec<Axis>,
    pub interest: MouseInterest,
}

impl std::fmt::Debug for LayoutNodes {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("LayoutNodes")
            .field("nodes", &ArenaDebug(&self.nodes))
            .finish()
    }
}

impl LayoutNodes {
    pub fn current_axis(&self) -> Option<Axis> {
        self.axis_stack.iter().nth_back(1).copied()
    }

    pub fn get(&self, id: ViewId) -> Option<&LayoutNode> {
        self.nodes.get(id.0)
    }

    #[inline(always)]
    pub fn compute(
        &mut self,
        nodes: &ViewNodes,
        input: &InputState,
        stylesheet: &mut Stylesheet,
        id: ViewId,
        space: Space,
    ) -> Size {
        nodes.begin(id);
        stylesheet.swap(id);

        self.nodes.insert_at(id.0, LayoutNode::default());
        let (size, interest) = nodes
            .scoped(id, |node| {
                let axis = node.primary_axis();

                self.axis_stack.push(axis);
                let layout = Layout {
                    nodes,
                    layout: self,
                    input,
                    stylesheet,
                };
                let size = node.layout(layout, space);
                self.axis_stack.pop();

                let interest = node.interests();
                (size, interest)
            })
            .unwrap();

        let new_layer = self.interest.current_layer_root() == Some(id);
        if interest.is_mouse_any() {
            self.interest.insert(id, interest);
        }
        if new_layer {
            self.interest.pop_layer();
        }

        let clipping_enabled = self.clip_stack.last() == Some(&id);

        let clipped_by = if clipping_enabled {
            self.clip_stack.iter().nth_back(2).copied()
        } else {
            self.clip_stack.last().copied()
        };

        {
            let layout = &mut self.nodes[id.0];
            layout.clipping_enabled = clipping_enabled;
            layout.new_layer = new_layer;
            layout.clipped_by = clipped_by;
            layout.interest = interest;
            layout.rect.set_size(size);
        };

        if clipping_enabled {
            self.clip_stack.pop();
        }

        stylesheet.swap(id);
        nodes.end(id);

        size
    }

    pub fn intrinsic_size(&self, nodes: &ViewNodes, id: ViewId, axis: Axis, extent: f32) -> f32 {
        nodes.begin(id);

        let size = nodes
            .scoped(id, |node| {
                let size = IntrinsicSize {
                    nodes,
                    layout: self,
                };
                node.size(size, axis, extent)
            })
            .unwrap();

        nodes.end(id);
        size
    }

    pub fn enable_clipping(&mut self, nodes: &ViewNodes) {
        self.clip_stack.push(nodes.current());
    }

    pub fn new_layer(&mut self, nodes: &ViewNodes) {
        self.interest.push_layer(nodes.current());
    }

    pub fn set_position(&mut self, id: ViewId, pos: impl Into<Pos2>) {
        if let Some(node) = self.nodes.get_mut(id.0) {
            let offset = pos.into().to_vec2();
            node.rect = node.rect.translate(offset);
        }
    }

    pub fn set_size(&mut self, id: ViewId, size: impl Into<Vec2>) {
        if let Some(node) = self.nodes.get_mut(id.0) {
            node.rect.set_size(size);
        }
    }
}

impl LayoutNodes {
    const fn new() -> Self {
        Self {
            nodes: Arena::new(),
            clip_stack: Vec::new(),
            axis_stack: Vec::new(),
            interest: MouseInterest::new(),
        }
    }

    #[profiling::function]
    fn compute_all(
        &mut self,
        nodes: &ViewNodes,
        input: &InputState,
        stylesheet: &mut Stylesheet,
        rect: Rect,
    ) {
        let space = Space::from_size(rect.size().into()).loosen();
        self.compute(nodes, input, stylesheet, nodes.root(), space);
        self.resolve(nodes, rect);
    }

    #[profiling::function]
    fn resolve(&mut self, nodes: &ViewNodes, rect: Rect) {
        let mut queue = VecDeque::from_iter([(nodes.root(), rect.left_top())]);
        while let Some((id, pos)) = queue.pop_front() {
            let Some(layout) = self.nodes.get_mut(id.0) else {
                continue;
            };

            let Some(node) = nodes.get(id) else {
                continue;
            };

            if layout.rect.is_empty() {
                continue;
            }

            let offset = pos.to_vec2();
            layout.rect = layout.rect.translate(offset);
            queue.extend(node.children.iter().map(|&id| (id, layout.rect.min)))
        }
    }
}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct ViewId(pub(crate) thunderdome::Index);

impl std::hash::Hash for ViewId {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        state.write_u64(self.0.to_bits());
    }
}

impl std::fmt::Debug for ViewId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}v{}", self.0.slot(), self.0.generation())
    }
}

#[derive(Default)]
pub struct LayoutNode {
    pub rect: Rect,
    pub clipping_enabled: bool,
    pub new_layer: bool,
    pub clipped_by: Option<ViewId>,
    pub interest: Interest,
}

impl std::fmt::Debug for LayoutNode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("LayoutNode")
            .field(
                "rect",
                &format_args!(
                    "{{{}, {} .. {}, {}}}",
                    self.rect.min.x, self.rect.min.y, self.rect.max.y, self.rect.max.y
                ),
            )
            .field("clipping_enabled", &self.clipping_enabled)
            .field("new_layer", &self.new_layer)
            .field("clipped_by", &self.clipped_by)
            .field("interest", &self.interest)
            .finish()
    }
}

pub struct ViewNode {
    pub parent: Option<ViewId>,
    pub children: Vec<ViewId>,
    pub(in crate::view) view: RefCell<Slot>,
    pub(in crate::view) next: usize,
}

impl std::fmt::Debug for ViewNode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ViewNode")
            .field("parent", &self.parent)
            .field("children", &self.children)
            .field("next", &self.next)
            .finish()
    }
}

#[derive(Default)]
pub enum Slot {
    #[default]
    Vacant,
    Inhabited(Box<dyn Erased>),
}

impl std::fmt::Debug for Slot {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Vacant => write!(f, "Vacant"),
            Self::Inhabited(view) => view.fmt(f),
        }
    }
}

impl Slot {
    pub fn new(view: impl View + 'static) -> Self {
        Self::Inhabited(Box::new(view))
    }

    pub fn give(&mut self, view: Box<dyn Erased>) {
        assert!(matches!(self, Self::Vacant));
        *self = Self::Inhabited(view)
    }

    pub fn take(&mut self) -> Box<dyn Erased> {
        let Self::Inhabited(view) = std::mem::take(self) else {
            unreachable!("slot was vacant")
        };
        view
    }
}

impl std::ops::Deref for Slot {
    type Target = Box<dyn Erased>;
    #[inline(always)]
    #[track_caller]
    fn deref(&self) -> &Self::Target {
        let Self::Inhabited(view) = self else {
            unreachable!("slot was vacant")
        };
        view
    }
}

impl std::ops::DerefMut for Slot {
    #[inline(always)]
    #[track_caller]
    fn deref_mut(&mut self) -> &mut Self::Target {
        let Self::Inhabited(view) = self else {
            unreachable!("slot was vacant")
        };
        view
    }
}

#[derive(Debug)]
struct Root;

impl View for Root {
    type Args<'v> = ();
    type Response = ();

    fn create(_: Self::Args<'_>) -> Self {
        Self
    }

    fn primary_axis(&self) -> Axis {
        Axis::Vertical
    }

    fn layout(&mut self, mut layout: Layout, space: Space) -> Size {
        layout.new_layer();
        self.default_layout(layout, space.loosen());
        space.max
    }
}
