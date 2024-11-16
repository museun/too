use std::collections::VecDeque;

use slotmap::SecondaryMap;

use crate::{
    layout::{Axis, Flex},
    math::{Pos2, Rect, Size, Space, Vec2},
};

use super::{input::InputState, Interest, ViewId, ViewNodes};

pub struct Layout<'a> {
    pub nodes: &'a ViewNodes,
    pub layout: &'a mut LayoutNodes,
    pub input: &'a InputState,
    pub current: ViewId,
}

impl<'a> Layout<'a> {
    pub fn compute(&mut self, id: ViewId, space: Space) -> Size {
        self.layout.compute(self.nodes, self.input, id, space)
    }

    pub fn parent_axis(&self) -> Axis {
        self.layout.current_axis().unwrap()
    }

    pub fn flex(&self, id: ViewId) -> Flex {
        self.nodes.get(id).unwrap().view.borrow().flex()
    }

    pub fn set_layer(&mut self, layer: Layer) {
        self.layout.set_layer(self.current, layer);
    }

    pub fn size(&self, id: ViewId) -> Size {
        self.layout
            .get(id)
            .map(|c| c.rect.size().into())
            .unwrap_or_default()
    }

    pub fn intrinsic_size(&self, id: ViewId, axis: Axis, extent: f32) -> f32 {
        self.layout.intrinsic_size(self.nodes, id, axis, extent)
    }

    pub fn new_layer(&mut self) {
        self.layout.new_layer(self.nodes);
    }

    pub fn enable_clipping(&mut self) {
        self.layout.enable_clipping(self.nodes);
    }

    pub fn remove(&mut self, id: ViewId) {
        self.layout.remove(id);
        self.input.remove(id);
    }

    pub fn set_position(&mut self, id: ViewId, pos: impl Into<Pos2>) {
        self.layout.set_position(id, pos);
    }

    pub fn set_size(&mut self, id: ViewId, size: impl Into<Vec2>) {
        self.layout.set_size(id, size)
    }
}

pub struct IntrinsicSize<'a> {
    pub nodes: &'a ViewNodes,
    pub layout: &'a LayoutNodes,
}

impl<'a> IntrinsicSize<'a> {
    pub fn size(&self, id: ViewId, axis: Axis, extent: f32) -> f32 {
        self.layout.intrinsic_size(self.nodes, id, axis, extent)
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

    fn insert(&mut self, id: ViewId, interest: Interest) {
        self.stack
            .last()
            .and_then(|&(_, index)| self.layers.get_mut(index))
            .unwrap()
            .push((id, interest));
    }

    pub fn iter(&self) -> impl Iterator<Item = (ViewId, Interest)> + '_ {
        self.layers
            .iter()
            .rev()
            .flat_map(|layer| layer.iter().copied())
    }
}

#[derive(Default, Debug)]
pub struct LayoutNodes {
    pub(super) nodes: SecondaryMap<ViewId, LayoutNode>,
    clip_stack: Vec<ViewId>,
    axis_stack: Vec<Axis>,
    pub(super) interest: MouseInterest,
}

impl LayoutNodes {
    pub fn current_axis(&self) -> Option<Axis> {
        self.axis_stack.iter().nth_back(1).copied()
    }

    pub fn get(&self, id: ViewId) -> Option<&LayoutNode> {
        self.nodes.get(id)
    }

    pub fn contains(&self, id: ViewId) -> bool {
        self.nodes.contains_key(id)
    }

    pub fn rect(&self, id: ViewId) -> Option<Rect> {
        self.get(id).map(|c| c.rect)
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
        let id = nodes.current();
        self.interest.push_layer(id);
    }

    pub fn remove(&mut self, id: ViewId) {
        self.nodes.remove(id);
    }

    pub fn set_position(&mut self, id: ViewId, pos: impl Into<Pos2>) {
        if let Some(node) = self.nodes.get_mut(id) {
            let offset = pos.into().to_vec2();
            node.rect = node.rect.translate(offset);
        }
    }

    pub fn set_size(&mut self, id: ViewId, size: impl Into<Vec2>) {
        if let Some(node) = self.nodes.get_mut(id) {
            node.rect.set_size(size);
        }
    }
}

impl LayoutNodes {
    pub(super) fn new() -> Self {
        Self {
            nodes: SecondaryMap::new(),
            clip_stack: Vec::new(),
            axis_stack: Vec::new(),
            interest: MouseInterest::new(),
        }
    }

    #[inline(always)]
    pub(super) fn compute(
        &mut self,
        nodes: &ViewNodes,
        input: &InputState,
        id: ViewId,
        space: Space,
    ) -> Size {
        nodes.begin(id);

        self.nodes.insert(id, LayoutNode::new(id));
        let (size, interest) = nodes
            .scoped(id, |node| {
                self.axis_stack.push(node.primary_axis());
                let layout = Layout {
                    nodes,
                    layout: self,
                    input,
                    current: id,
                };
                let size = node.layout(layout, space);
                self.axis_stack.pop();
                (size, node.interests())
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
            let layout = &mut self.nodes[id];
            layout.clipping_enabled = clipping_enabled;
            layout.new_layer = new_layer;
            layout.clipped_by = clipped_by;
            layout.interest = interest;
            layout.rect.set_size(size);
        };

        if clipping_enabled {
            self.clip_stack.pop();
        }

        nodes.end(id);

        size
    }

    pub(super) fn set_layer(&mut self, current: ViewId, layer: Layer) {
        self.nodes[current].layer = layer;
    }

    pub(crate) fn begin(&mut self) {
        self.clip_stack.clear();
        self.axis_stack.clear();
    }

    pub(super) fn end(&mut self) {
        self.interest.clear();
    }

    #[cfg_attr(feature = "profile", profiling::function)]
    pub(super) fn compute_all(&mut self, nodes: &ViewNodes, input: &InputState, rect: Rect) {
        let space = Space::from_size(rect.size().into()).loosen();
        self.compute(nodes, input, nodes.root(), space);
        self.resolve(nodes, rect);
    }

    #[cfg_attr(feature = "profile", profiling::function)]
    pub(super) fn resolve(&mut self, nodes: &ViewNodes, rect: Rect) {
        let mut queue = VecDeque::from([(nodes.root(), rect.left_top())]);
        while let Some((id, pos)) = queue.pop_front() {
            let Some(layout) = self.nodes.get_mut(id) else {
                continue;
            };

            let Some(node) = nodes.get(id) else {
                continue;
            };

            if layout.rect.is_empty() {
                continue;
            }

            // we can't clamp for things -x or -y
            if !layout.rect.min.x.is_negative() && !layout.rect.min.y.is_negative() {
                layout.rect.min = layout.rect.min.clamp(rect.min, rect.max);
            }
            layout.rect.max = layout.rect.max.clamp(rect.min, rect.max);
            layout.rect = layout.rect.translate(pos.to_vec2());

            queue.extend(node.children.iter().map(|&id| (id, layout.rect.min)))
        }
    }
}

#[derive(Copy, Clone, Default, Debug, PartialEq, PartialOrd)]
pub enum Layer {
    Bottom,
    #[default]
    Middle,
    Top,
    Debug,
}

#[derive(Default)]
pub struct LayoutNode {
    pub id: ViewId,
    pub rect: Rect,
    pub layer: Layer,
    pub new_layer: bool,
    pub clipping_enabled: bool,
    pub clipped_by: Option<ViewId>,
    pub interest: Interest,
}

impl LayoutNode {
    pub const fn new(id: ViewId) -> Self {
        Self {
            id,
            rect: Rect::ZERO,
            layer: Layer::Middle,
            new_layer: false,
            clipping_enabled: false,
            clipped_by: None,
            interest: Interest::NONE,
        }
    }
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
