use crate::{
    layout::Axis,
    math::{Pos2, Vec2},
};

use super::{
    geom::{Flex, Size, Space},
    input::InputState,
    state::{LayoutNodes, ViewId, ViewNodes},
};

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
