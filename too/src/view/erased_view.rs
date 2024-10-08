use std::{borrow::Cow, marker::PhantomData};

use super::{
    geom::{Size, Space},
    input::{Event, EventCtx, Handled},
    view::View,
    AnimateCtx, DestroyCtx, DrawCtx, Interest, LayoutCtx,
};

pub trait ErasedView: std::any::Any {
    type State: 'static;
    fn interest(&self) -> Interest;
    fn event(&mut self, ctx: EventCtx<Self::State>, event: &Event) -> Handled;
    fn animate(&mut self, ctx: AnimateCtx<Self::State>, dt: f32);
    fn layout(&mut self, ctx: LayoutCtx<Self::State>, space: Space) -> Size;
    fn draw(&mut self, ctx: DrawCtx<Self::State>);
    fn destroy(&mut self, ctx: DestroyCtx<Self::State>);
    fn type_name(&self) -> Cow<'static, str>;
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any;
}

pub struct ViewMarker<T: 'static, V: View<T>> {
    pub view: V,
    _marker: PhantomData<T>,
}

impl<T: 'static, V: View<T>> ViewMarker<T, V> {
    pub const fn new(view: V) -> Self {
        Self {
            view,
            _marker: PhantomData,
        }
    }
}

impl<T: 'static, V: View<T> + 'static> ErasedView for ViewMarker<T, V> {
    type State = T;

    fn interest(&self) -> Interest {
        <V as View<T>>::interest(&self.view)
    }

    fn event(&mut self, ctx: EventCtx<Self::State>, event: &Event) -> Handled {
        <V as View<T>>::event(&mut self.view, ctx, event)
    }

    fn animate(&mut self, ctx: AnimateCtx<Self::State>, dt: f32) {
        <V as View<T>>::animate(&mut self.view, ctx, dt);
    }

    fn layout(&mut self, ctx: LayoutCtx<Self::State>, space: Space) -> Size {
        <V as View<T>>::layout(&mut self.view, ctx, space)
    }

    fn draw(&mut self, ctx: DrawCtx<Self::State>) {
        <V as View<T>>::draw(&mut self.view, ctx);
    }

    fn destroy(&mut self, ctx: DestroyCtx<Self::State>) {
        <V as View<T>>::destroy(&mut self.view, ctx);
    }

    fn type_name(&self) -> Cow<'static, str> {
        V::short_name()
    }

    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        &mut self.view as _
    }
}
