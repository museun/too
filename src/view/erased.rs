use crate::{
    layout::{Axis, Flex},
    math::{Size, Space},
};

use super::{
    builder::ViewMarker, EventCtx, Handled, Interest, IntrinsicSize, Layout, Render, View,
    ViewEvent,
};

pub trait Erased: std::any::Any + std::fmt::Debug + ViewMarker {
    fn interests(&self) -> Interest;

    fn flex(&self) -> Flex;
    fn interactive(&self) -> bool;

    fn size(&self, size: IntrinsicSize, axis: Axis, extent: f32) -> f32;
    fn primary_axis(&self) -> Axis;

    fn event(&mut self, event: ViewEvent, ctx: EventCtx) -> Handled;
    fn layout(&mut self, layout: Layout, space: Space) -> Size;
    fn draw(&mut self, render: Render);

    fn as_mut_any(&mut self) -> &mut dyn std::any::Any;
    fn type_name(&self) -> &'static str;
}

impl<T: View + ViewMarker> Erased for T {
    #[inline(always)]
    fn interests(&self) -> Interest {
        T::interests(self)
    }

    #[inline(always)]
    fn flex(&self) -> Flex {
        T::flex(self)
    }

    #[inline(always)]
    fn interactive(&self) -> bool {
        T::interactive(self)
    }

    #[inline(always)]
    fn event(&mut self, event: ViewEvent, ctx: EventCtx) -> Handled {
        T::event(self, event, ctx)
    }

    #[inline(always)]
    fn size(&self, size: IntrinsicSize, axis: Axis, extent: f32) -> f32 {
        T::size(self, size, axis, extent)
    }

    #[inline(always)]
    fn primary_axis(&self) -> Axis {
        T::primary_axis(self)
    }

    #[inline(always)]
    fn layout(&mut self, layout: Layout, space: Space) -> Size {
        T::layout(self, layout, space)
    }

    #[inline(always)]
    fn draw(&mut self, render: Render) {
        T::draw(self, render)
    }

    fn as_mut_any(&mut self) -> &mut dyn std::any::Any {
        self
    }
    fn type_name(&self) -> &'static str {
        std::any::type_name::<T>()
    }
}
