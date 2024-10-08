use crate::animation::AnimationManager;

use super::{Properties, ViewId};

pub struct DestroyCtx<'a, T: 'static> {
    pub current_id: ViewId,
    pub children: &'a [ViewId],
    pub animations: &'a mut AnimationManager,
    pub properties: &'a mut Properties,
    pub(super) _marker: std::marker::PhantomData<T>,
}

impl<'a, T: 'static> DestroyCtx<'a, T> {
    pub fn destroy(&mut self, child: ViewId) {

        // let ctx = DestroyCtx {
        //     current_id: child,
        //     children: &node.children,
        //     animations: self.animations,
        //     properties: self.properties,
        //     nodes: self.nodes,
        // };

        // node.view.destroy(ctx);
        // self.nodes[child.0].inhabit(node);
    }
}
