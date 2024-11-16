use crate::{
    layout::{Axis, Flex},
    math::{Size, Space},
};

use super::{EventCtx, Handled, Interest, IntrinsicSize, Layout, Render, Response, Ui, ViewEvent};

pub trait Builder<'v>: Sized {
    type View: View<Args<'v> = Self>;
}

pub trait ViewExt<'v>: Builder<'v> {
    fn show(self, ui: &Ui) -> Response<<Self::View as View>::Response> {
        ui.show(self)
    }

    fn show_children<R>(
        self,
        ui: &Ui,
        show: impl FnOnce(&Ui) -> R,
    ) -> Response<(<Self::View as View>::Response, R)>
    where
        R: 'static,
    {
        ui.show_children(self, show)
    }
}

impl<'v, T> ViewExt<'v> for T where T: Builder<'v> {}

#[allow(unused_variables)]
pub trait View: Sized + 'static + std::fmt::Debug {
    type Args<'v>;
    type Response: 'static + Default;

    fn create(args: Self::Args<'_>) -> Self;

    fn update(&mut self, args: Self::Args<'_>, ui: &Ui) -> Self::Response {
        *self = Self::create(args);
        Self::Response::default()
    }

    fn flex(&self) -> Flex {
        Flex::Loose(0.0)
    }

    fn interests(&self) -> Interest {
        Interest::NONE
    }

    fn primary_axis(&self) -> Axis {
        Axis::Horizontal
    }

    fn event(&mut self, event: ViewEvent, ctx: EventCtx) -> Handled {
        self.default_event(event, ctx)
    }

    fn size(&self, intrinsic: IntrinsicSize, axis: Axis, extent: f32) -> f32 {
        let node = intrinsic.nodes.get_current();
        let mut size = 0.0_f32;
        for &child in &node.children {
            size = size.max(intrinsic.size(child, axis, extent))
        }
        size
    }

    fn layout(&mut self, layout: Layout, space: Space) -> Size {
        self.default_layout(layout, space)
    }

    fn draw(&mut self, render: Render) {
        self.default_draw(render)
    }

    fn default_event(&mut self, event: ViewEvent, mut ctx: EventCtx) -> Handled {
        let node = ctx.nodes.get_current();
        let mut resp = Handled::Bubble;
        for &child in &node.children {
            let new = ctx.event(child, event);
            if new.is_sink() {
                return new;
            }
            resp = new;
        }
        resp
    }

    fn default_layout(&mut self, mut layout: Layout, space: Space) -> Size {
        let current = layout.nodes.get_current();
        let mut size = Size::ZERO;
        for &child in &current.children {
            size = size.max(layout.compute(child, space))
        }
        size
    }

    fn default_draw(&mut self, mut render: Render) {
        let current = render.nodes.get_current();
        for &child in &current.children {
            render.draw(child)
        }
    }
}
