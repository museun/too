use std::collections::VecDeque;

use crate::math::Rect;

use super::{
    erased::Erased, layout::Properties, Builder, InputState, LayoutNode, LayoutNodes, ViewId,
    ViewNodes,
};

/// A filter depth
#[derive(Copy, Clone, Debug, Default, PartialEq)]
pub enum Depth {
    /// Only visit the immeditate children
    Immediate,
    /// Only visit children up to specified depth
    Specific(usize),
    #[default]
    /// Visit all children
    All,
}

pub trait Filterable {
    fn filter(&self) -> Filter<'_>;
}

pub struct Filter<'a> {
    nodes: &'a ViewNodes,
    layout: &'a LayoutNodes,
    input: &'a InputState,
}

impl<'a> Filter<'a> {
    pub const fn new(nodes: &'a ViewNodes, layout: &'a LayoutNodes, input: &'a InputState) -> Self {
        Self {
            nodes,
            layout,
            input,
        }
    }

    pub fn by_region(&self, start: ViewId, depth: Depth, region: Rect) -> Vec<ViewId> {
        self.filter_layout(start, depth, |_, layout| region.contains_rect(layout.rect))
    }

    pub fn by_property(
        &self,
        start: ViewId,
        depth: Depth,
        mut filter: impl FnMut(Properties) -> bool,
    ) -> Vec<ViewId> {
        self.filter_layout(start, depth, |_, layout| {
            filter(layout.properties(self.input))
        })
    }

    pub fn lookup<'v, T, R>(
        &self,
        id: ViewId,
        mut found: impl FnMut(&<T as Builder<'v>>::View) -> R,
    ) -> Option<R>
    where
        T: Builder<'v>,
        R: 'static,
    {
        let node = self.find_type::<T>(id, crate::view::Depth::All)?;
        let mut out = None;
        self.nodes.scoped(node, |erased| {
            if let Some(view) = erased.as_any().downcast_ref::<T::View>() {
                out = Some(found(view))
            }
        });
        out
    }

    pub fn find_type<'v, T>(&self, start: ViewId, depth: Depth) -> Option<ViewId>
    where
        T: Builder<'v>,
    {
        self.find_first(start, depth, |_id, erased, _layout| {
            erased.as_any().downcast_ref::<T::View>().is_some()
        })
    }

    pub fn by_type<'v, T>(&self, start: ViewId, depth: Depth) -> Vec<ViewId>
    where
        T: Builder<'v>,
    {
        self.find_childern(start, depth, |_id, erased, _layout| {
            erased.as_any().downcast_ref::<T::View>().is_some()
        })
    }

    pub fn filter_layout(
        &self,
        start: ViewId,
        depth: Depth,
        mut filter: impl FnMut(ViewId, &LayoutNode) -> bool,
    ) -> Vec<ViewId> {
        self.find_childern(start, depth, |id, _, layout| {
            let Some(layout) = layout else { return false };
            filter(id, layout)
        })
    }

    pub(crate) fn find_first(
        &self,
        start: ViewId,
        depth: Depth,
        mut filter: impl FnMut(ViewId, &dyn Erased, Option<&LayoutNode>) -> bool,
    ) -> Option<ViewId> {
        let mut out = None;
        self.visit(start, |id, d| {
            if !Self::depth_check(depth, d) {
                return false;
            }

            if let Some(true) = self
                .nodes
                .scoped(id, |erased| filter(id, erased, self.layout.get(id)))
            {
                out = Some(id);
                return false;
            };

            true
        });
        out
    }

    pub(crate) fn find_childern(
        &self,
        start: ViewId,
        depth: Depth,
        mut filter: impl FnMut(ViewId, &dyn Erased, Option<&LayoutNode>) -> bool,
    ) -> Vec<ViewId> {
        let mut out = vec![];

        self.visit(start, |id, d| {
            if !Self::depth_check(depth, d) {
                return false;
            }
            if let Some(true) = self
                .nodes
                .scoped(id, |erased| filter(id, erased, self.layout.get(id)))
            {
                out.push(id);
            };
            true
        });

        out
    }

    fn depth_check(depth: Depth, current: usize) -> bool {
        match depth {
            Depth::Immediate if current > 0 => false,
            Depth::Specific(s) if s <= current => false,
            _ => true,
        }
    }

    fn visit(&self, mut start: ViewId, mut child: impl FnMut(ViewId, usize) -> bool) {
        let Some(node) = self.nodes.get(start) else {
            return;
        };

        let mut depth = 0;

        // check to see if the start node was the one they wanted
        if !child(start, depth) {
            return;
        }

        let mut queue = VecDeque::from_iter(node.children.iter().copied());

        // the only way the get/parent can fail is if the 'start' node has a child of the root
        // be its impossible to form a DAG with our tree, nothing can be acylic
        //
        // getting the parent node cannot fail because we always start the walk
        // with the start node's children
        //
        // and because we checked if the start node exists, none of the children can be missing
        while let Some(id) = queue.pop_front() {
            // because this is operating on view nodes and starting at the children
            // this child will always exist

            // we always start at the children of the parent
            // so the parent will always exist
            let parent = self.nodes.get(id).unwrap().parent.unwrap();
            if parent != start {
                start = parent;
                depth += 1;
            }

            if !child(id, depth) {
                break;
            }

            let node = self.nodes.get(id).unwrap();
            let iter = node.children.iter().copied();
            queue.extend(iter);
        }
    }
}
