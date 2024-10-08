use crate::overlay::DebugOverlay;

use super::{
    geom::{Rectf, Size, Space, Vector},
    input::Input,
    Node, Properties, ViewId,
};

pub struct LayoutCtx<'a, T: 'static> {
    pub current_id: ViewId,
    pub children: &'a [ViewId],
    pub state: &'a mut T,
    pub properties: &'a mut Properties,

    pub(super) client_rect: Rectf,
    pub(super) input: &'a mut Input,
    pub(super) nodes: &'a mut thunderdome::Arena<Node<T>>,
    pub(super) stack: &'a mut Vec<ViewId>,
    pub(super) debug: &'a mut DebugOverlay,
}

impl<'a, T: 'static> LayoutCtx<'a, T> {
    pub fn compute_layout(&mut self, child: ViewId, space: Space) -> Size {
        let Some(node) = self.nodes.get_mut(child.0) else {
            return Size::ZERO;
        };

        let Some(mut node) = node.take() else {
            unreachable!("node: {child:?} was missing")
        };

        self.stack.push(child);

        let size = node.view.layout(
            LayoutCtx {
                current_id: child,
                children: &node.children,
                client_rect: self.client_rect,
                properties: self.properties,
                state: self.state,
                input: self.input,
                nodes: self.nodes,
                stack: self.stack,
                debug: self.debug,
            },
            space,
        );

        let is_new_mouse_layer = self.input.mouse.current_layer_root() == Some(child);
        let is_new_keyboard_layer = self.input.keyboard.current_layer_root() == Some(child);

        let interest = node.view.interest();
        if interest.is_mouse_any() {
            self.input.mouse.add(child, interest);
        }
        if interest.is_key_input() {
            self.input.keyboard.add(child);
        }

        if is_new_mouse_layer {
            self.input.mouse.pop_layer();
        }
        if is_new_keyboard_layer {
            self.input.keyboard.pop_layer();
        }

        node.rect = Rectf::from(size.clamp(Size::ZERO, self.client_rect.size()));
        node.interest = interest;

        assert_eq!(Some(child), self.stack.pop());
        self.nodes[child.0].inhabit(node);

        size
    }

    pub fn new_layer_for(&mut self, id: ViewId) {
        self.input.mouse.push_layer(id);
        self.input.keyboard.push_layer(id);
    }

    pub fn new_layer(&mut self) {
        self.new_layer_for(self.current_id);
    }

    pub fn translate_pos(&mut self, child: ViewId, offset: impl Into<Vector>) {
        if let Some(node) = self.nodes.get_mut(child.0) {
            node.rect += offset.into();
        }
    }

    pub fn translate_size(&mut self, child: ViewId, size: impl Into<Size>) {
        if let Some(node) = self.nodes.get_mut(child.0) {
            node.rect += size.into()
        }
    }

    pub fn debug(&mut self, msg: impl ToString) {
        self.debug.push(msg.to_string());
    }
}
