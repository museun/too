use crate::{animation::AnimationManager, overlay::DebugOverlay, Pixel};

use super::{
    geom::{float_step_exclusive, Point, Rectf, Vector},
    Node, Properties, Theme, ViewId,
};

pub struct Surface<'a> {
    pub(super) rect: Rectf,
    pub(super) surface: &'a mut crate::Surface,
}

impl<'a> Surface<'a> {
    pub fn surface_raw(&mut self) -> &mut crate::Surface {
        self.surface
    }

    pub const fn rect(&self) -> Rectf {
        self.rect
    }

    pub fn horizontal_fill(&mut self, vec: impl Into<Vector>, pixel: impl Into<Pixel>) {
        let vec = vec.into().round();
        let pixel = pixel.into();
        for x in float_step_exclusive(vec.x, vec.y, 1.0) {
            self.set((x, 0.0), pixel);
        }
    }

    pub fn vertical_fill(&mut self, vec: impl Into<Vector>, pixel: impl Into<Pixel>) {
        let vec = vec.into().round();
        let pixel = pixel.into();
        for y in float_step_exclusive(vec.x, vec.y, 1.0) {
            self.set((0.0, y), pixel);
        }
    }

    pub fn fill(&mut self, pixel: impl Into<Pixel>) {
        let pixel = pixel.into();
        let vec = Vector::from((self.rect.width(), self.rect.height())).round();
        for y in float_step_exclusive(0.0, vec.y, 1.0) {
            for x in float_step_exclusive(0.0, vec.x, 1.0) {
                self.set((x, y), pixel);
            }
        }
    }

    pub fn set(&mut self, point: impl Into<Point>, pixel: impl Into<Pixel>) {
        let point = point.into() + self.rect.left_top().to_vector();
        let pos = point.into();
        self.surface.set(pos, pixel.into());
    }
}

pub struct DrawCtx<'a, 't, T: 'static> {
    pub rect: Rectf,
    pub current_id: ViewId,
    pub children: &'a [ViewId],
    pub surface: Surface<'t>,
    pub state: &'a mut T,
    pub theme: &'a Theme,
    pub properties: &'a mut Properties,
    pub animations: &'a mut AnimationManager,

    pub(super) nodes: &'a mut thunderdome::Arena<Node<T>>,
    pub(super) stack: &'a mut Vec<ViewId>,
    pub(super) debug: &'a mut DebugOverlay,
}

impl<'a, 'c: 't, 't, T: 'static> DrawCtx<'a, 't, T> {
    pub fn draw(&mut self, id: ViewId) {
        let Some(node) = self.nodes.get_mut(id.0) else {
            return;
        };

        let Some(mut node) = node.take() else {
            unreachable!("node: {:?} was missing", id)
        };

        self.stack.push(id);

        let ctx = DrawCtx {
            rect: node.rect,
            current_id: id,
            children: &node.children,
            surface: Surface {
                rect: node.rect,
                surface: self.surface.surface,
            },
            state: self.state,
            theme: self.theme,
            properties: self.properties,
            animations: self.animations,
            nodes: self.nodes,
            stack: self.stack,
            debug: self.debug,
        };

        node.view.draw(ctx);
        assert_eq!(Some(id), self.stack.pop());
        self.nodes[id.0].inhabit(node);
    }

    pub fn debug(&mut self, msg: impl ToString) {
        self.debug.push(msg.to_string());
    }
}
