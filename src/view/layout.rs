use std::collections::VecDeque;

use slotmap::{Key as _, SecondaryMap};

use crate::{
    layout::{Axis, Flex},
    math::{Pos2, Rect, Size, Space, Vec2},
};

use super::{input::InputState, Filter, Filterable, Interest, ViewId, ViewNodes};

impl<'a> Filterable for Layout<'a> {
    fn filter(&self) -> super::Filter<'_> {
        Filter::new(self.nodes, self.layout, self.input)
    }
}

/// Layout context
///
/// Typically. you'll:
/// - get your current node
/// - get your children
/// - iterate over them
/// - call compute with their id
/// - calculate some total size based on the above
/// - return that size
///
/// ## In your code:
/// ```compile_fail
/// let current = layout.nodes.get_current();
/// let mut size = Size::ZERO;
/// for &child in &current.children {
///     size = size.max(layout.compute(child, space))
/// }
/// size
/// ```
pub struct Layout<'a> {
    /// Immutable access to the node tree.
    ///
    /// You can get your current view with [`nodes.get_current()`](ViewNodes::get_current)
    ///
    /// From your current node you can get:
    /// - your children with [`node.children`](crate::view::ViewNode::children)
    /// - your parent with  [`node.parent`](crate::view::ViewNode::parent)
    ///
    pub nodes: &'a ViewNodes,
    /// Mutable access to the layout tree.
    pub layout: &'a mut LayoutNodes,
    /// Mutable access to the input state tree.
    pub input: &'a mut InputState,
    /// The current id of the view that is computing its layout
    pub current: ViewId,
}

impl<'a> Layout<'a> {
    /// Compute the layout size of a view with the provided space
    pub fn compute(&mut self, id: ViewId, space: Space) -> Size {
        self.layout.compute(self.nodes, self.input, id, space)
    }

    /// Get the axis of your parent
    pub fn parent_axis(&self) -> Axis {
        self.layout.current_axis().unwrap()
    }

    /// Get the flex factor of a view
    pub fn flex(&self, id: ViewId) -> Flex {
        self.nodes.get(id).unwrap().view.borrow().flex()
    }

    /// Get the computed size of a view
    pub fn size(&self, id: ViewId) -> Size {
        self.layout
            .get(id)
            .map(|c| c.rect.size().into())
            .unwrap_or_default()
    }

    /// Get the instrinic size of a view, with the provided axis and extent
    pub fn intrinsic_size(&self, id: ViewId, axis: Axis, extent: f32) -> f32 {
        self.layout.intrinsic_size(self.nodes, id, axis, extent)
    }

    pub fn new_layer(&mut self) {
        self.layout.new_layer(self.nodes);
    }

    pub fn set_layer(&mut self, layer: Layer) {
        self.layout.set_layer(self.current, layer);
    }

    /// Enables clipping.
    ///
    /// Clipping constraints your rendering rect to your computed size.
    ///
    /// You can use this if you produce an infinite size to ensure you can't draw outside of your visible rect.
    pub fn enable_clipping(&mut self) {
        self.layout.enable_clipping(self.nodes);
    }

    /// Remove a view from the layout and input trees
    pub fn remove(&mut self, id: ViewId) {
        self.layout.remove(id);
        self.input.remove(id);
    }

    /// Position a view in the layout tree.
    ///
    /// By default, your children are positioned at your origin (top-left) of your rect.
    ///
    /// If you need to place them somewhere else, you can use this. The `pos` is relative to your local rect.
    ///
    /// E.g. pos2(5, 3) is your top_left + (5, 3)
    pub fn set_position(&mut self, id: ViewId, pos: impl Into<Pos2>) {
        self.layout.set_position(id, pos);
    }

    /// Set the size of a view in the layout tree.
    ///
    /// This lets you override a views computed size
    pub fn set_size(&mut self, id: ViewId, size: impl Into<Vec2>) {
        self.layout.set_size(id, size)
    }

    /// Get the properties for the current node
    pub fn properties(&self) -> Properties {
        self.properties_for(self.current).unwrap()
    }

    /// Get the properties for a specific node
    pub fn properties_for(&self, id: ViewId) -> Option<Properties> {
        self.layout.get(id).map(|c| c.properties(self.input))
    }

    pub fn filter(&self) -> Filter<'_> {
        <Self as Filterable>::filter(self)
    }
}

/// Context for calculating the intrinstic size of a view
pub struct IntrinsicSize<'a> {
    pub nodes: &'a ViewNodes,
    pub layout: &'a LayoutNodes,
}

impl<'a> IntrinsicSize<'a> {
    /// Calculate the intrinsic size for a view, with the provided access and extent.
    ///
    /// The `extent` is the 'length' of the axis. (e.g. height or width)
    pub fn size(&self, id: ViewId, axis: Axis, extent: f32) -> f32 {
        self.layout.intrinsic_size(self.nodes, id, axis, extent)
    }
}

#[derive(Default, Debug)]
pub struct EventInterest {
    layers: Vec<Vec<(ViewId, Interest)>>,
    stack: Vec<(ViewId, usize)>,
}

impl EventInterest {
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

/// The tree for the layouts of all of the views
#[derive(Default)]
pub struct LayoutNodes {
    pub(super) nodes: SecondaryMap<ViewId, LayoutNode>,
    clip_stack: Vec<ViewId>,
    axis_stack: Vec<Axis>,
    pub(super) interest: EventInterest,
}

impl std::fmt::Debug for LayoutNodes {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        struct NodeDebug<'a>(&'a SecondaryMap<ViewId, LayoutNode>);
        impl<'a> std::fmt::Debug for NodeDebug<'a> {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                f.debug_map()
                    .entries(self.0.iter().map(|(k, v)| (k.data(), v)))
                    .finish()
            }
        }

        f.debug_struct("ViewNodes")
            .field("nodes", &NodeDebug(&self.nodes))
            .finish()
    }
}

impl LayoutNodes {
    pub(super) fn current_axis(&self) -> Option<Axis> {
        self.axis_stack.iter().nth_back(1).copied()
    }

    /// Get a [`LayoutNode`] by id
    pub fn get(&self, id: ViewId) -> Option<&LayoutNode> {
        self.nodes.get(id)
    }

    /// Does this tree contain that id?
    pub fn contains(&self, id: ViewId) -> bool {
        self.nodes.contains_key(id)
    }

    /// Get a [`Rect`] by id
    ///
    /// If this id was not found this'll return None
    ///
    /// If the rect for the id was 'empty' this'll return None.
    ///
    /// The latter can happen if you try to get a rect for a view before its layout has ever happened.
    ///
    /// Or it can happen if the view's size was set to zero (essentially hiding it).
    pub fn rect(&self, id: ViewId) -> Option<Rect> {
        self.get(id).map(|c| c.rect).filter(|c| !c.is_empty())
    }

    pub(super) fn intrinsic_size(
        &self,
        nodes: &ViewNodes,
        id: ViewId,
        axis: Axis,
        extent: f32,
    ) -> f32 {
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

    pub(super) fn enable_clipping(&mut self, nodes: &ViewNodes) {
        self.clip_stack.push(nodes.current());
    }

    pub(super) fn new_layer(&mut self, nodes: &ViewNodes) {
        let id = nodes.current();
        self.interest.push_layer(id);
    }

    pub(super) fn remove(&mut self, id: ViewId) {
        self.nodes.remove(id);
    }

    pub(super) fn set_position(&mut self, id: ViewId, pos: impl Into<Pos2>) {
        if let Some(node) = self.nodes.get_mut(id) {
            let offset = pos.into().to_vec2();
            node.rect = node.rect.translate(offset);
        }
    }

    pub(super) fn set_size(&mut self, id: ViewId, size: impl Into<Vec2>) {
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
            interest: EventInterest::new(),
        }
    }

    #[inline(always)]
    pub(super) fn compute(
        &mut self,
        nodes: &ViewNodes,
        input: &mut InputState,
        id: ViewId,
        space: Space,
    ) -> Size {
        nodes.begin(id);

        self.nodes.insert(id, LayoutNode::new(id));
        let (size, interest, interactive) = nodes
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
                (size, node.interests(), node.interactive())
            })
            .unwrap();

        let new_layer = self.interest.current_layer_root() == Some(id);
        if !interest.is_none() {
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
            layout.interactive = interactive;
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
    pub(super) fn compute_all(&mut self, nodes: &ViewNodes, input: &mut InputState, rect: Rect) {
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

/// A layer a view should be on, relative to its parent
#[derive(Copy, Clone, Default, Debug, PartialEq, PartialOrd)]
pub enum Layer {
    /// At the bottom layer relative to its siblings
    Bottom,
    #[default]
    /// The default layer
    Middle,
    /// Above other siblings on lower layers
    Top,
    /// Above other siblings on lower layers
    Debug,
}

/// Properties for a layout node
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Properties {
    /// Is this node interactive?
    pub interactive: bool,
    /// Is this node hovered?
    pub hovered: bool,
    /// Is this node focused?
    pub focused: bool,
    /// The interests of this node
    pub interests: Interest,
}

/// A node in the layout tree for a view
#[derive(Default, Debug)]
pub struct LayoutNode {
    /// The id of the view
    pub id: ViewId,
    /// The computed rectangle for the view
    pub rect: Rect,
    /// Which layer the view is on
    pub layer: Layer,
    /// Was this view on a new layer?
    pub new_layer: bool,
    /// Is the view being clipped?
    pub clipping_enabled: bool,
    /// Who is clipping the view
    pub clipped_by: Option<ViewId>,
    /// The event interests of the view
    pub interest: Interest,
    /// Is this node interactive?
    pub interactive: bool,
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
            interactive: false,
        }
    }

    pub(crate) fn properties(&self, state: &InputState) -> Properties {
        Properties {
            interactive: self.interactive,
            focused: state.is_focused(self.id),
            hovered: state.is_hovered(self.id),
            interests: self.interest,
        }
    }
}
