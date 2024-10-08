use crate::animation::AnimationManager;

use super::{Node, ViewId};

pub struct AnimateCtx<'a, T: 'static> {
    pub current_id: ViewId,
    pub children: &'a [ViewId],
    pub state: &'a mut T,
    pub animations: &'a mut AnimationManager,
    // TODO this needs the rect (but is it valid here?)
    pub(super) nodes: &'a mut thunderdome::Arena<Node<T>>,
}

impl<'a, T: 'static> AnimateCtx<'a, T> {
    pub fn animate(&mut self, id: ViewId, dt: f32) {
        let Some(node) = self.nodes.get_mut(id.0) else {
            return;
        };

        let Some(mut node) = node.take() else {
            unreachable!("node: {:?} was missing", id)
        };

        let ctx = AnimateCtx {
            current_id: id,
            children: &node.children,
            state: self.state,
            animations: self.animations,
            nodes: self.nodes,
        };

        node.view.animate(ctx, dt);
        self.nodes[id.0].inhabit(node);
    }
}
