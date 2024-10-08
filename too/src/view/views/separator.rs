use crate::{layout::Axis, view::UpdateCtx, Pixel};

use super::super::{
    elements::Elements,
    geom::{Size, Space},
    properties::FilledProperty,
    view::Context,
    DrawCtx, LayoutCtx, View, ViewExt,
};

#[derive(Copy, Clone, Default, Debug, PartialEq)]
pub struct Separator {
    axis: Axis,
}

impl Separator {
    pub const fn horizontal() -> Self {
        Self::axis(Axis::Horizontal)
    }
    pub const fn vertical() -> Self {
        Self::axis(Axis::Vertical)
    }
    pub const fn axis(axis: Axis) -> Self {
        Self { axis }
    }

    pub fn show<T: 'static>(self, ctx: &mut Context<T>) {
        SeparatorView::show(self.axis, ctx);
    }
}

impl FilledProperty for SeparatorView {
    const FILLED: char = Elements::HORIZONTAL_LINE;
    const CROSS: char = Elements::VERTICAL_LINE;
}

struct SeparatorView {
    axis: Axis,
}

impl<T: 'static> View<T> for SeparatorView {
    type Args<'a> = Axis;
    type Response = ();

    fn create(args: Self::Args<'_>) -> Self {
        Self { axis: args }
    }

    fn update(&mut self, ctx: UpdateCtx<T>, args: Self::Args<'_>) -> Self::Response {
        self.axis = args
    }

    fn layout(&mut self, ctx: LayoutCtx<T>, space: Space) -> Size {
        match self.axis {
            Axis::Horizontal => Size::new(space.max.width, 1.0),
            Axis::Vertical => Size::new(1.0, space.max.height),
        }
    }

    fn draw(&mut self, mut ctx: DrawCtx<T>) {
        let filled = match self.axis {
            Axis::Horizontal => ctx.properties.filled::<Self>(),
            Axis::Vertical => ctx.properties.filled_cross::<Self>(),
        };
        let pixel = Pixel::new(filled).fg(ctx.theme.surface);
        match self.axis {
            Axis::Horizontal => ctx.surface.horizontal_fill((0.0, ctx.rect.width()), pixel),
            Axis::Vertical => ctx.surface.vertical_fill((0.0, ctx.rect.height()), pixel),
        }
    }
}

pub fn vertical_separator<T>(ctx: &mut Context<T>) {
    Separator::vertical().show(ctx)
}

pub fn horizontal_separator<T>(ctx: &mut Context<T>) {
    Separator::horizontal().show(ctx)
}
